#include "UiBridge.h"
#include <QCoreApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QRegularExpression>
#include <QThread>
#include <QtEndian>
#include <limits>
namespace {
constexpr int MaxFrame = 65536;
constexpr int MaxWire = MaxFrame * 129;
} // namespace
UiBridge::UiBridge(QObject *parent, int capacity)
    : QObject(parent), m_capacity(qBound(1, capacity, 128)) {
  m_clock.start();
  m_watchdog.setInterval(100);
  connect(&m_watchdog, &QTimer::timeout, this, [this] {
    assertGui();
    if (!m_ready && m_process && m_clock.elapsed() - m_started > 60000) {
      fail("UI_STARTUP_TIMEOUT");
      stop();
      return;
    }
    for (auto it = m_pending.cbegin(); it != m_pending.cend(); ++it) {
      if (m_clock.elapsed() - it->since > 5000) {
        fail("UI_OPERATION_TIMEOUT");
        stop();
        break;
      }
    }
  });
  m_watchdog.start();
}
UiBridge::~UiBridge() { stop(); }
void UiBridge::assertGui() const {
  const bool ok = QThread::currentThread() == qApp->thread();
  m_threadOk = m_threadOk && ok;
  if (!ok)
    qFatal("UI_GUI_THREAD_REQUIRED");
}
void UiBridge::fail(const QString &code) {
  assertGui();
  m_error = code;
  emit errorChanged();
}
void UiBridge::setDarkMode(bool dark) {
  assertGui();
  m_dark = dark;
  emit themeChanged();
}
void UiBridge::setTextScale(qreal scale) {
  assertGui();
  m_scale = qBound(1.0, scale, 2.0);
  emit themeChanged();
}
QString UiBridge::committedValue(const QString &id) const {
  return m_values.value(id).toString();
}
bool UiBridge::hasDraft(const QString &id) const {
  return m_drafts.contains(id);
}
void UiBridge::setLocalDraft(const QString &id, const QString &text,
                             const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch()) {
    ++m_rejected;
    return;
  }
  bool allowed = false;
  for (const auto &v : m_controls) {
    const auto c = v.toMap();
    if (c.value("id").toString() == id &&
        c.value("type").toString() != "readout") {
      allowed = true;
      break;
    }
  }
  if (!allowed) {
    fail("UI_BINDING_MISMATCH");
    return;
  }
  if (text.size() > 512 ||
      m_draftClock == std::numeric_limits<quint64>::max()) {
    fail("UI_QUOTA_EXCEEDED");
    return;
  }
  m_drafts[id] = text;
  m_draftVersions[id] = ++m_draftClock;
}
void UiBridge::edit(const QString &id, const QString &text,
                    const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch()) {
    ++m_rejected;
    return;
  }
  setLocalDraft(id, text, epoch);
  send(QJsonObject{
      {"type", "edit"}, {"controlId", id}, {"proposedValue", text}});
}
void UiBridge::apply(const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch()) {
    ++m_rejected;
    return;
  }
  QJsonObject values;
  for (auto it = m_drafts.cbegin(); it != m_drafts.cend(); ++it)
    values.insert(it.key(), it.value());
  send(QJsonObject{{"type", "apply"}, {"proposedValues", values}});
}
void UiBridge::save() { send(QJsonObject{{"type", "save"}}); }
void UiBridge::start(const LaunchConfig &c) {
  assertGui();
  stop();
  m_error.clear();
  emit errorChanged();
  m_expectedRoom = c.room;
  m_started = m_clock.elapsed();
  const quint64 epoch = m_epoch;
  auto *p = new QProcess(this);
  m_process = p;
  QPointer<UiBridge> guard(this);
  QPointer<QProcess> process(p);
  connect(p, &QProcess::readyReadStandardOutput, this, [guard, process, epoch] {
    if (guard && process && guard->m_process == process &&
        guard->m_epoch == epoch)
      guard->read(process, epoch);
  });
  connect(p, &QProcess::readyReadStandardError, this, [guard, process, epoch] {
    if (!guard || !process || guard->m_epoch != epoch)
      return;
    const auto previous = process->readChannel();
    process->setReadChannel(QProcess::StandardError);
    qint64 total = 0;
    while (process->bytesAvailable() > 0) {
      total += process->read(4096).size();
      if (total > 4096) {
        guard->fail("UI_DIAGNOSTIC_LIMIT");
        process->kill();
        break;
      }
    }
    process->setReadChannel(previous);
  });
  connect(p, qOverload<int, QProcess::ExitStatus>(&QProcess::finished), this,
          [guard, process, epoch](int code, QProcess::ExitStatus) {
            if (guard && process && guard->m_epoch == epoch && code != 0)
              guard->fail("UI_UNAVAILABLE");
          });
  connect(p, &QProcess::errorOccurred, this,
          [guard, epoch](QProcess::ProcessError) {
            if (guard && guard->m_epoch == epoch)
              guard->fail("UI_UNAVAILABLE");
          });
  p->setProgram(c.worker);
  p->setArguments({c.source, c.store, c.python, c.semantic, c.abi, c.room,
                   c.restore ? "restore" : "new"});
  p->start();
}
void UiBridge::stop() {
  assertGui();
  ++m_epoch;
  QProcess *p = m_process;
  m_process = nullptr;
  if (p) {
    p->disconnect(this);
    p->closeWriteChannel();
    if (!p->waitForFinished(1500)) {
      p->kill();
      p->waitForFinished(500);
    }
    p->deleteLater();
  }
  m_buffer.clear();
  m_pending.clear();
  m_controls.clear();
  m_values.clear();
  m_drafts.clear();
  m_draftVersions.clear();
  m_key = {};
  m_lastProjection = {};
  m_revision = 0;
  m_generation = 0;
  m_sequence = 0;
  m_ready = false;
  emit readyChanged();
  emit documentChanged();
  emit valuesChanged();
}
bool UiBridge::counter(const QJsonValue &value, quint64 &number) {
  if (!value.isString()) {
    return false;
  }
  const QString s = value.toString();
  static const QRegularExpression pattern("\\A(?:0|[1-9][0-9]{0,19})\\z");
  if (!pattern.match(s).hasMatch()) {
    return false;
  }
  bool ok = false;
  number = s.toULongLong(&ok);
  return ok;
}
void UiBridge::send(const QJsonObject &command) {
  assertGui();
  if (!m_ready || !m_process) {
    fail("UI_UNAVAILABLE");
    return;
  }
  if (m_pending.size() >= m_capacity) {
    fail("QUEUE_FULL");
    return;
  }
  if (m_request == std::numeric_limits<quint64>::max()) {
    fail("UI_QUOTA_EXCEEDED");
    return;
  }
  const QString token = QString::number(++m_request);
  QJsonObject request{{"protocol", "0.1"},
                      {"localToken", token},
                      {"key", m_key},
                      {"expectedRoomRevision", QString::number(m_revision)},
                      {"expectedGeneration", QString::number(m_generation)},
                      {"command", command}};
  const QByteArray body = QJsonDocument(request).toJson(QJsonDocument::Compact);
  if (body.size() > MaxFrame ||
      m_process->bytesToWrite() + body.size() + 4 > MaxWire) {
    fail("QUEUE_FULL");
    return;
  }
  QByteArray frame(4, '\0');
  qToBigEndian<quint32>(body.size(), reinterpret_cast<uchar *>(frame.data()));
  frame += body;
  m_pending[token] = Pending{m_draftVersions, m_clock.elapsed()};
  if (m_process->write(frame) != frame.size()) {
    m_pending.remove(token);
    fail("UI_TRANSPORT_ERROR");
  }
}
void UiBridge::read(QProcess *p, quint64 epoch) {
  assertGui();
  if (p->bytesAvailable() > MaxWire) {
    fail("UI_QUOTA_EXCEEDED");
    p->kill();
    return;
  }
  while (p->bytesAvailable() > 0) {
    const int need =
        m_buffer.size() < 4
            ? 4 - m_buffer.size()
            : 4 +
                  int(qFromBigEndian<quint32>(
                      reinterpret_cast<const uchar *>(m_buffer.constData()))) -
                  m_buffer.size();
    if (need < 0 || need > MaxFrame) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    m_buffer += p->read(need);
    if (m_buffer.size() < 4)
      return;
    const quint32 size = qFromBigEndian<quint32>(
        reinterpret_cast<const uchar *>(m_buffer.constData()));
    if (size == 0 || size > MaxFrame) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    if (m_buffer.size() < int(size) + 4)
      continue;
    QJsonParseError error;
    const QJsonDocument doc = QJsonDocument::fromJson(m_buffer.mid(4), &error);
    m_buffer.clear();
    if (error.error != QJsonParseError::NoError || !doc.isObject()) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    accept(doc.object(), epoch);
  }
}
void UiBridge::accept(const QJsonObject &frame, quint64 epoch) {
  assertGui();
  if (epoch != m_epoch) {
    ++m_rejected;
    return;
  }
  if (frame.value("protocol") != "0.1") {
    fail("INVALID_UI_FRAME");
    return;
  }
  const QString kind = frame.value("kind").toString();
  if (kind == "unavailable" || kind == "transportError" ||
      kind == "backpressure") {
    m_pending.remove(frame.value("localToken").toString());
    fail(frame.value("code").toString());
    return;
  }
  if (kind != "ready" && kind != "projection" && kind != "saved") {
    fail("INVALID_UI_FRAME");
    return;
  }
  const QJsonObject p = frame.value("projection").toObject();
  const QJsonObject key = p.value("key").toObject();
  quint64 revision, generation, sequence;
  if (!counter(p.value("roomRevision"), revision) ||
      !counter(p.value("executionGeneration"), generation) ||
      !counter(p.value("projectionSequence"), sequence)) {
    fail("INVALID_UI_COUNTER");
    return;
  }
  if (kind == "ready") {
    if (key.value("roomId").toString() != m_expectedRoom ||
        key.value("instanceId").toString() != "component" ||
        key.value("uiId").toString().isEmpty()) {
      fail("UI_OWNERSHIP_DENIED");
      return;
    }
    if (m_ready) {
      ++m_rejected;
      return;
    }
    const QJsonObject document = frame.value("document").toObject();
    const QJsonObject root = document.value("root").toObject();
    const auto controls = root.value("children").toArray();
    if (document.value("uiVersion") != "0.1" || root.value("type") != "form" ||
        controls.isEmpty() || controls.size() > 32) {
      fail("INVALID_UI");
      return;
    }
    for (const auto &v : controls) {
      const QString type = v.toObject().value("type").toString();
      if (type != "decimal-input" && type != "integer-input" &&
          type != "readout") {
        fail("INVALID_UI");
        return;
      }
    }
    m_key = key;
    m_title = root.value("title").toString();
    m_controls = controls.toVariantList();
    m_ready = true;
    emit documentChanged();
    emit readyChanged();
  } else if (!m_ready || key != m_key || revision < m_revision ||
             generation < m_generation || sequence <= m_sequence) {
    ++m_rejected;
    return;
  }
  m_revision = revision;
  m_generation = generation;
  m_sequence = sequence;
  m_values = p.value("values").toObject().toVariantMap();
  m_lastProjection = p;
  ++m_applied;
  const QString token = frame.value("localToken").toString();
  const auto pending = m_pending.take(token);
  if (frame.value("accepted").toBool()) {
    for (auto it = pending.versions.cbegin(); it != pending.versions.cend();
         ++it) {
      if (m_draftVersions.value(it.key()) == it.value()) {
        m_drafts.remove(it.key());
        m_draftVersions.remove(it.key());
      }
    }
  }
  const auto diagnostics = p.value("diagnostics").toArray();
  m_error = diagnostics.isEmpty()
                ? QString()
                : diagnostics.first().toObject().value("code").toString();
  emit errorChanged();
  emit valuesChanged();
  emit projectionApplied();
  if (kind == "saved")
    emit saved();
}
