#include "DesktopBridge.h"
#include <QCoreApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QRegularExpression>
#include <QThread>
#include <QtEndian>
#include <limits>
namespace {
constexpr int FrameLimit = 1024 * 1024;
constexpr int WireLimit = FrameLimit * 33;
} // namespace
DesktopBridge::DesktopBridge(QObject *parent, int capacity)
    : QObject(parent), m_capacity(qBound(1, capacity, 32)) {
  m_clock.start();
  m_watchdog.setInterval(100);
  connect(&m_watchdog, &QTimer::timeout, this, [this] {
    assertGui();
    if (m_quitting && m_clock.elapsed() - m_quitStarted > 15000) {
      fail("SHUTDOWN_TIMEOUT");
      m_process->kill();
      return;
    }
    if (m_process && m_process->state() != QProcess::NotRunning &&
        m_lastState.isEmpty() && m_clock.elapsed() - m_started > 60000) {
      fail("UI_STARTUP_TIMEOUT");
      m_process->kill();
    }
    for (auto it = m_pending.cbegin(); it != m_pending.cend(); ++it)
      if (m_clock.elapsed() - it->since > 15000) {
        fail("UI_OPERATION_TIMEOUT");
        m_process->kill();
        break;
      }
  });
  m_watchdog.start();
}
DesktopBridge::~DesktopBridge() {
  assertGui();
  if (m_process) {
    m_process->disconnect(this);
    m_process->kill();
    m_process->deleteLater();
  }
}
void DesktopBridge::assertGui() const {
  const bool ok = QThread::currentThread() == qApp->thread();
  m_threadOk = m_threadOk && ok;
  if (!ok)
    qFatal("GUI_THREAD_REQUIRED");
}
void DesktopBridge::fail(const QString &code) {
  assertGui();
  m_error = code;
  emit errorChanged();
  emit stateChanged();
}
void DesktopBridge::setDarkMode(bool value) {
  assertGui();
  m_dark = value;
  emit themeChanged();
}
void DesktopBridge::setTextScale(qreal value) {
  assertGui();
  m_scale = qBound(1.0, value, 2.0);
  emit themeChanged();
}
QString DesktopBridge::committedValue(const QString &id) const {
  return m_values.value(id).toString();
}
bool DesktopBridge::hasDraft(const QString &id) const {
  return m_drafts.contains(id);
}
void DesktopBridge::setLocalDraft(const QString &id, const QString &text,
                                  const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch() || !ready()) {
    ++m_rejected;
    return;
  }
  bool allowed = false;
  for (const auto &v : m_controls) {
    const auto c = v.toMap();
    if (c.value("id") == id && c.value("type") != "readout") {
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
  m_versions[id] = ++m_draftClock;
  emit stateChanged();
}
void DesktopBridge::edit(const QString &id, const QString &text,
                         const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch() || !ready()) {
    ++m_rejected;
    return;
  }
  setLocalDraft(id, text, epoch);
  if (!m_versions.contains(id))
    return;
  sendUi({{"type", "edit"}, {"controlId", id}, {"proposedValue", text}},
         {{id, m_versions.value(id)}});
}
void DesktopBridge::apply(const QString &epoch) {
  assertGui();
  if (epoch != viewEpoch() || !ready()) {
    ++m_rejected;
    return;
  }
  QJsonObject values;
  for (auto it = m_drafts.cbegin(); it != m_drafts.cend(); ++it)
    values.insert(it.key(), it.value());
  sendUi({{"type", "apply"}, {"proposedValues", values}}, m_versions);
}
void DesktopBridge::save() {
  assertGui();
  if (ready())
    apply(viewEpoch());
}
void DesktopBridge::sendUi(const QJsonObject &command,
                           QMap<QString, quint64> versions) {
  QJsonObject request{{"protocol", "0.1"},
                      {"localToken", QString::number(m_request + 1)},
                      {"key", m_key},
                      {"expectedRoomRevision", QString::number(m_revision)},
                      {"expectedGeneration", QString::number(m_generation)},
                      {"command", command}};
  send({{"type", "ui"}, {"request", request}}, false, versions);
}
void DesktopBridge::selection(const QJsonObject &command) {
  assertGui();
  if (m_quitting || m_switching)
    return;
  const auto token = send(command, true);
  if (token.isEmpty())
    return;
  ++m_viewEpoch;
  m_pending[token].view = m_viewEpoch;
  m_switching = true;
  m_drafts.clear();
  m_versions.clear();
  emit readyChanged();
  emit stateChanged();
}
void DesktopBridge::createRoom(const QJsonObject &pin) {
  selection({{"type", "create"}, {"pin", pin}});
}
void DesktopBridge::selectRoom(const QString &id) {
  selection({{"type", "select"}, {"roomId", id}});
}
void DesktopBridge::setPinned(bool pinned) {
  assertGui();
  if (ready())
    send({{"type", "pin"}, {"pinned", pinned}});
}
void DesktopBridge::quit() {
  assertGui();
  if (m_quitting)
    return;
  if (!m_process || m_process->state() == QProcess::NotRunning) {
    emit shutdownComplete(m_pending.isEmpty() && m_quitAck);
    return;
  }
  // Allow already accepted commands to finish; queue quit only after
  // acknowledgments.
  m_quitting = true;
  m_quitStarted = m_clock.elapsed();
  emit readyChanged();
  emit stateChanged();
  continueQuit();
}
QString DesktopBridge::send(const QJsonObject &command, bool selection,
                            QMap<QString, quint64> versions) {
  assertGui();
  if (!m_process || m_process->state() != QProcess::Running ||
      (m_quitting && command.value("type") != "quit")) {
    fail("UI_UNAVAILABLE");
    return {};
  }
  if (m_pending.size() >= m_capacity) {
    fail("QUEUE_FULL");
    return {};
  }
  if (m_request == std::numeric_limits<quint64>::max()) {
    fail("UI_QUOTA_EXCEEDED");
    return {};
  }
  const QString token = QString::number(++m_request);
  const QByteArray bytes =
      QJsonDocument(QJsonObject{{"protocol", "desktop.1"},
                                {"token", token},
                                {"epoch", QString::number(m_hostEpoch)},
                                {"command", command}})
          .toJson(QJsonDocument::Compact);
  if (bytes.size() > FrameLimit ||
      m_process->bytesToWrite() + bytes.size() + 4 > WireLimit) {
    fail("QUEUE_FULL");
    return {};
  }
  QByteArray frame(4, '\0');
  qToBigEndian<quint32>(bytes.size(), reinterpret_cast<uchar *>(frame.data()));
  frame += bytes;
  m_pending[token] =
      Pending{m_viewEpoch, versions, m_clock.elapsed(), selection};
  if (m_process->write(frame) != frame.size()) {
    m_pending.remove(token);
    fail("UI_TRANSPORT_ERROR");
    return {};
  }
  emit busyChanged();
  return token;
}
void DesktopBridge::start(const DesktopConfig &c) {
  assertGui();
  if (m_process) {
    fail("ALREADY_STARTED");
    return;
  }
  m_started = m_clock.elapsed();
  auto *p = new QProcess(this);
  m_process = p;
  QPointer<DesktopBridge> guard(this);
  QPointer<QProcess> child(p);
  connect(p, &QProcess::readyReadStandardOutput, this, [guard, child] {
    if (guard && child && guard->m_process == child)
      guard->read();
  });
  connect(p, &QProcess::readyReadStandardError, this, [guard, child] {
    if (!guard || !child)
      return;
    const auto previous = child->readChannel();
    child->setReadChannel(QProcess::StandardError);
    qint64 size = 0;
    while (child->bytesAvailable() > 0) {
      size += child->read(4096).size();
      if (size > 4096) {
        guard->fail("UI_DIAGNOSTIC_LIMIT");
        child->kill();
        break;
      }
    }
    child->setReadChannel(previous);
  });
  connect(p, qOverload<int, QProcess::ExitStatus>(&QProcess::finished), this,
          [guard, child](int code, QProcess::ExitStatus status) {
            if (!guard || !child)
              return;
            guard->assertGui();
            if (guard->m_quitting) {
              emit guard->shutdownComplete(guard->m_quitAck && code == 0 &&
                                           status == QProcess::NormalExit);
            } else {
              guard->m_ready = false;
              guard->fail("WORKSPACE_UNAVAILABLE");
              emit guard->readyChanged();
            }
          });
  connect(p, &QProcess::errorOccurred, this, [guard](QProcess::ProcessError) {
    if (guard)
      guard->fail("WORKSPACE_UNAVAILABLE");
  });
  connect(p, &QProcess::started, this, [this, p] {
    assertGui();
    m_workerPid = p->processId();
  });
  p->setProgram(c.worker);
  p->setArguments({c.registry, c.workspace, c.python, c.semantic, c.abi});
  p->start();
}
bool DesktopBridge::counter(const QJsonValue &value, quint64 &n) {
  if (!value.isString())
    return false;
  static const QRegularExpression re("\\A(?:0|[1-9][0-9]{0,19})\\z");
  if (!re.match(value.toString()).hasMatch())
    return false;
  bool ok = false;
  n = value.toString().toULongLong(&ok);
  return ok;
}
void DesktopBridge::read() {
  assertGui();
  auto *p = m_process;
  if (p->bytesAvailable() > WireLimit) {
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
    if (need < 0 || need > FrameLimit) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    m_buffer += p->read(need);
    if (m_buffer.size() < 4)
      return;
    const quint32 length = qFromBigEndian<quint32>(
        reinterpret_cast<const uchar *>(m_buffer.constData()));
    if (length == 0 || length > FrameLimit) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    if (m_buffer.size() < int(length) + 4)
      continue;
    QJsonParseError error;
    const auto document = QJsonDocument::fromJson(m_buffer.mid(4), &error);
    m_buffer.clear();
    if (error.error != QJsonParseError::NoError || !document.isObject()) {
      fail("INVALID_UI_FRAME");
      p->kill();
      return;
    }
    accept(document.object());
  }
}
void DesktopBridge::accept(const QJsonObject &frame) {
  assertGui();
  if (frame.value("protocol") != "desktop.1") {
    fail("INVALID_UI_FRAME");
    return;
  }
  const QString kind = frame.value("kind").toString();
  const QString token = frame.value("token").toString();
  if (kind == "backpressure" || kind == "transportError" || kind == "fatal") {
    m_pending.remove(token);
    fail(frame.value("code").toString());
    emit busyChanged();
    if (kind == "fatal") {
      m_ready = false;
      emit readyChanged();
    }
    if (token == m_quitToken && m_quitting) {
      m_quitting = false;
      m_quitToken.clear();
      fail("SHUTDOWN_NOT_ACKNOWLEDGED");
      emit readyChanged();
    } else
      continueQuit();
    return;
  }
  if (kind != "state") {
    fail("INVALID_UI_FRAME");
    return;
  }
  const bool initial = token.isEmpty() && m_lastState.isEmpty();
  if (!initial && !m_pending.contains(token)) {
    ++m_rejected;
    return;
  }
  const Pending pending = m_pending.value(token);
  if (!initial && pending.view != m_viewEpoch) {
    m_pending.remove(token);
    emit busyChanged();
    ++m_rejected;
    emit commandFinished(token, false);
    return;
  }
  quint64 epoch;
  if (!counter(frame.value("epoch"), epoch) || epoch < m_hostEpoch) {
    fail("INVALID_UI_COUNTER");
    return;
  }
  if (m_switching && !pending.select) {
    ++m_rejected;
    return;
  }
  const bool changed = initial || pending.select;
  const QJsonObject projection = frame.value("projection").toObject();
  quint64 revision = 0, generation = epoch, sequence = 0;
  if (!projection.isEmpty()) {
    if (!counter(projection.value("roomRevision"), revision) ||
        !counter(projection.value("executionGeneration"), generation) ||
        !counter(projection.value("projectionSequence"), sequence)) {
      fail("INVALID_UI_COUNTER");
      return;
    }
    if (!changed &&
        (projection.value("key").toObject() != m_key || revision < m_revision ||
         generation < m_generation || sequence <= m_sequence)) {
      ++m_rejected;
      return;
    }
    if (projection.value("key").toObject().value("roomId") !=
            frame.value("selectedRoom") ||
        projection.value("key").toObject().value("instanceId") != "component") {
      fail("UI_OWNERSHIP_DENIED");
      return;
    }
  }
  if (frame.value("installed").toArray().size() > 128 ||
      frame.value("rooms").toArray().size() > 16) {
    fail("UI_QUOTA_EXCEEDED");
    return;
  }
  if (changed) {
    const auto root =
        frame.value("document").toObject().value("root").toObject();
    const auto children = root.value("children").toArray();
    if (!projection.isEmpty() && (root.value("type") != "form" ||
                                  children.isEmpty() || children.size() > 32)) {
      fail("INVALID_UI");
      return;
    }
    for (const auto &v : children) {
      const QString type = v.toObject().value("type").toString();
      if (type != "decimal-input" && type != "integer-input" &&
          type != "readout") {
        fail("INVALID_UI");
        return;
      }
    }
    m_controls = children.toVariantList();
    m_title = root.value("title").toString();
    m_selected = frame.value("selectedRoom").toString();
    m_switching = false;
  }
  m_pending.remove(token);
  emit busyChanged();
  m_hostEpoch = epoch;
  m_revision = revision;
  m_generation = generation;
  m_sequence = sequence;
  m_key = projection.value("key").toObject();
  m_values = projection.value("values").toObject().toVariantMap();
  m_installed = frame.value("installed").toArray().toVariantList();
  m_rooms = frame.value("rooms").toArray().toVariantList();
  const bool accepted = frame.value("accepted").toBool();
  if (accepted) {
    for (auto it = pending.versions.cbegin(); it != pending.versions.cend();
         ++it)
      if (m_versions.value(it.key()) == it.value()) {
        m_drafts.remove(it.key());
        m_versions.remove(it.key());
      }
  }
  if (accepted) {
    // Earlier incomplete submissions may now be part of this committed
    // aggregate. Exact equality is already committed, while unrelated different
    // drafts stay.
    for (const auto &id : m_drafts.keys())
      if (m_values.contains(id) &&
          m_values.value(id).toString() == m_drafts.value(id)) {
        m_drafts.remove(id);
        m_versions.remove(id);
      }
  }
  m_error.clear();
  m_fieldErrors.clear();
  const auto diagnostics = frame.value("diagnostics").toArray();
  for (const auto &entry : diagnostics) {
    const auto d = entry.toObject();
    const QString id = d.value("controlId").toString();
    QString label;
    for (const auto &v : m_controls)
      if (v.toMap().value("id") == id)
        label = v.toMap().value("label").toString();
    QString message = d.value("message").toString();
    const QString code = d.value("code").toString();
    QString guidance;
    if (code == "INCOMPLETE_INPUT")
      guidance = "Complete this field before calculating.";
    else if (code == "SCHEMA_VIOLATION" && !id.isEmpty())
      guidance =
          "Check this field's format and allowed limits. Use plain decimal "
          "digits or a whole number, without currency symbols or grouping.";
    else if (code == "STATE_CONFLICT")
      guidance = "The room changed. Review your draft and choose Apply draft "
                 "to try again.";
    else if (code == "COMPONENT_TRAP" || code == "GUEST_ERROR" ||
             code == "SCHEMA_VIOLATION")
      guidance = "The component could not produce a valid result. The last "
                 "committed result is retained.";
    if (!guidance.isEmpty())
      message = guidance + " (" + message + ")";

    if (!label.isEmpty())
      message = label + ": " + message;
    if (!id.isEmpty())
      m_fieldErrors[id] = message;
    if (m_error.isEmpty())
      m_error = message;
  }
  if (!frame.value("recovery").isNull() &&
      !frame.value("recovery").toString().isEmpty())
    m_error = "Recovery required: " + frame.value("recovery").toString() +
              ". Original workspace retained.";
  m_ready = !projection.isEmpty() && frame.value("recovery").isNull();
  m_lastState = frame;
  if (changed)
    emit documentChanged();
  emit readyChanged();
  emit errorChanged();
  emit valuesChanged();
  emit stateChanged();
  emit commandFinished(token, accepted);
  if (token == m_quitToken && m_quitting) {
    m_quitAck = accepted;
    if (!accepted) {
      m_quitting = false;
      m_quitToken.clear();
      fail("SHUTDOWN_NOT_ACKNOWLEDGED");
      emit readyChanged();
    }
  }
  continueQuit();
}
void DesktopBridge::continueQuit() {
  assertGui();
  if (m_quitting && m_pending.isEmpty() && m_quitToken.isEmpty() && !m_quitAck)
    m_quitToken = send({{"type", "quit"}});
}
