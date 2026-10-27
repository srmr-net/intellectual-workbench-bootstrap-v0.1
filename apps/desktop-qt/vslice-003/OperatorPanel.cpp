#include "OperatorPanel.h"
#include <QApplication>
#include <QCheckBox>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QThread>
#include <QUuid>
#include <QVBoxLayout>
#include <QtEndian>
namespace {
QPushButton *button(const QString &text, const QString &name, QWidget *parent) {
  auto *b = new QPushButton(text, parent);
  b->setObjectName(name);
  b->setAccessibleName(text);
  return b;
}
} // namespace
OperatorPanel::OperatorPanel(DesktopShell &shell)
    : QWidget(shell.findChild<QWidget *>("activityContext")), m_shell(shell),
      m_process(shell.bridge().findChild<QProcess *>()) {
  setObjectName("operatorPanel");
  setAccessibleName("Request an installed tool");
  m_prefix = "op-" + QUuid::createUuid().toString(QUuid::Id128) + "-";
  auto *layout = new QVBoxLayout(this);
  auto *heading = new QLabel("Request an installed tool", this);
  heading->setTextFormat(Qt::PlainText);
  layout->addWidget(heading);
  m_intent = new QPlainTextEdit(this);
  m_intent->setObjectName("operatorIntent");
  m_intent->setAccessibleName("Tool request");
  m_intent->setPlaceholderText("Describe one tool and the values to use.");
  m_intent->setMaximumHeight(60);
  layout->addWidget(m_intent);
  m_prepare = button("Preview request", "operatorPrepare", this);
  layout->addWidget(m_prepare);
  m_submitted = new QPlainTextEdit(this);
  m_submitted->setObjectName("operatorSubmitted");
  m_submitted->setAccessibleName("Submitted user request");
  m_submitted->setReadOnly(true);
  m_submitted->setMaximumHeight(45);
  layout->addWidget(m_submitted);
  m_preview = new QPlainTextEdit(this);
  m_preview->setObjectName("operatorPreview");
  m_preview->setAccessibleName("Composition proposal preview");
  m_preview->setReadOnly(true);
  m_preview->setMaximumHeight(140);
  layout->addWidget(m_preview);
  m_consent = new QCheckBox("I approve this exact proposal", this);
  m_consent->setObjectName("operatorConsent");
  m_consent->setAccessibleName("Approve this exact proposal");
  layout->addWidget(m_consent);
  m_approve = button("Confirm and initialize", "operatorApprove", this);
  m_decline = button("Reject proposal", "operatorDecline", this);
  m_cancel = button("Cancel request", "operatorCancel", this);
  layout->addWidget(m_approve);
  layout->addWidget(m_decline);
  layout->addWidget(m_cancel);
  m_status = new QLabel("Preview a request before approving any change.", this);
  m_status->setObjectName("operatorStatus");
  m_status->setAccessibleName("Request status");
  m_status->setTextFormat(Qt::PlainText);
  m_status->setWordWrap(true);
  layout->addWidget(m_status);
  auto *context = qobject_cast<QVBoxLayout *>(parentWidget()->layout());
  context->insertWidget(context->count() - 1, this);
  m_deadline.setSingleShot(true);
  m_watchdog.setSingleShot(true);
  if (!m_process)
    qFatal("OPERATOR_WORKER_UNAVAILABLE");
  // Own only the new worker's bounded operator.1 stderr stream. The accepted
  // bridge retains stdin/stdout, finished/error handling and all renderer
  // paths.
  QObject::disconnect(m_process, SIGNAL(readyReadStandardError()),
                      &shell.bridge(), nullptr);
  connect(m_process, &QProcess::readyReadStandardError, this,
          &OperatorPanel::read);
  connect(m_process, qOverload<int, QProcess::ExitStatus>(&QProcess::finished),
          this, [this] {
            clearPreview();
            refresh();
          });
  connect(m_prepare, &QPushButton::clicked, this, [this] {
    const auto utterance = m_intent->toPlainText();
    if (utterance.isEmpty() || utterance.toUtf8().size() > 4096) {
      fail("UNSUPPORTED_REQUEST");
      return;
    }
    clearPreview();
    m_submitted->setPlainText(utterance);
    send({{"type", "prepare"}, {"utterance", utterance}});
  });
  connect(m_intent, &QPlainTextEdit::textChanged, this, [this] {
    if (!m_digest.isEmpty()) {
      clearPreview();
      m_status->setText("Request changed. Preview it again before approving.");
    }
  });
  connect(m_approve, &QPushButton::clicked, this, [this] {
    if (!m_consent->isChecked() || m_digest.isEmpty() || busy())
      return;
    const auto digest = m_digest;
    clearPreview();
    send({{"type", "approve"}, {"digest", digest}});
  });
  connect(m_decline, &QPushButton::clicked, this, [this] {
    clearPreview();
    send({{"type", "decline"}});
  });
  connect(m_cancel, &QPushButton::clicked, this, [this] {
    clearPreview();
    send({{"type", "cancel"}});
  });
  connect(m_consent, &QCheckBox::toggled, this, &OperatorPanel::refresh);
  connect(&shell.bridge(), &DesktopBridge::stateChanged, this,
          &OperatorPanel::refresh);
  connect(&m_deadline, &QTimer::timeout, this, [this] {
    clearPreview();
    fail("APPROVAL_EXPIRED");
  });
  connect(&m_watchdog, &QTimer::timeout, this, [this] {
    clearPreview();
    if (m_action == "approve") {
      fail("Approval outcome is pending. Reopen the workspace to inspect saved "
           "state if the host does not respond.");
      return;
    }
    if (m_pending.size() < 2)
      send({{"type", "cancel"}});
    fail("PROVIDER_TIMEOUT");
  });
  connect(m_process, &QProcess::started, this, &OperatorPanel::inspect);
  if (m_process->state() == QProcess::Running)
    QTimer::singleShot(0, this, &OperatorPanel::inspect);
  refresh();
}
void OperatorPanel::inspect() {
  if (!busy())
    send({{"type", "status"}});
}
QString OperatorPanel::error() const { return m_status->text(); }
bool OperatorPanel::guiThread() const {
  return QThread::currentThread() == qApp->thread();
}
void OperatorPanel::clearPreview() {
  m_deadline.stop();
  m_digest.clear();
  m_consent->setChecked(false);
  refresh();
}
void OperatorPanel::refresh() {
  const auto state = m_shell.bridge().lastState();
  const bool healthy = !state.isEmpty() && state.value("recovery").isNull() &&
                       m_process && m_process->state() == QProcess::Running;
  m_prepare->setEnabled(healthy && !busy());
  m_intent->setEnabled(healthy && !busy());
  m_consent->setEnabled(!m_digest.isEmpty() && !busy());
  m_approve->setEnabled(!m_digest.isEmpty() && m_consent->isChecked() &&
                        !busy());
  m_decline->setEnabled(!m_digest.isEmpty() && !busy());
  m_cancel->setEnabled(busy() && m_action == "prepare" && m_pending.size() < 2);
  if ((!state.value("recovery").isNull()) && !m_digest.isEmpty()) {
    m_digest.clear();
    m_deadline.stop();
    m_consent->setChecked(false);
    m_approve->setEnabled(false);
    m_decline->setEnabled(false);
    m_consent->setEnabled(false);
  }
}
void OperatorPanel::fail(const QString &code) {
  m_status->setText(code);
  refresh();
}
void OperatorPanel::send(const QJsonObject &command) {
  if (!m_process || m_process->state() != QProcess::Running ||
      m_pending.size() >= 2) {
    fail("QUEUE_FULL");
    return;
  }
  const auto token = m_prefix + QString::number(++m_token);
  const auto bytes = QJsonDocument(QJsonObject{{"protocol", "operator.1"},
                                               {"token", token},
                                               {"command", command}})
                         .toJson(QJsonDocument::Compact);
  if (bytes.size() > 20 * 1024) {
    fail("UNSUPPORTED_REQUEST");
    return;
  }
  QByteArray frame(4, '\0');
  qToBigEndian<quint32>(quint32(bytes.size()), frame.data());
  frame += bytes;
  m_current = token;
  m_action = command.value("type").toString();
  m_pending.insert(token);
  if (m_process->write(frame) != frame.size()) {
    m_pending.remove(token);
    m_current.clear();
    fail("PROVIDER_UNAVAILABLE");
    return;
  }
  m_watchdog.start(60000);
  m_status->setText("Waiting for the host…");
  refresh();
}
void OperatorPanel::read() {
  const auto saved = m_process->readChannel();
  m_process->setReadChannel(QProcess::StandardError);
  while (m_process->bytesAvailable() > 0) {
    m_buffer +=
        m_process->read(qMin<qint64>(4096, m_process->bytesAvailable()));
    if (m_buffer.size() > 65540) {
      m_buffer.clear();
      clearPreview();
      fail("PROVIDER_INVALID_OUTPUT");
      break;
    }
    while (m_buffer.size() >= 4) {
      const auto n = qFromBigEndian<quint32>(m_buffer.constData());
      if (n == 0 || n > 65536) {
        m_buffer.clear();
        clearPreview();
        fail("PROVIDER_INVALID_OUTPUT");
        break;
      }
      if (m_buffer.size() < qint64(n) + 4)
        break;
      const auto bytes = m_buffer.mid(4, int(n));
      m_buffer.remove(0, int(n) + 4);
      QJsonParseError error;
      const auto document = QJsonDocument::fromJson(bytes, &error);
      if (error.error != QJsonParseError::NoError || !document.isObject()) {
        clearPreview();
        fail("PROVIDER_INVALID_OUTPUT");
        continue;
      }
      accept(document.object());
    }
  }
  m_process->setReadChannel(saved);
}
void OperatorPanel::accept(const QJsonObject &reply) {
  if (reply.value("protocol") != "operator.1" ||
      reply.value("kind") != "reply") {
    clearPreview();
    fail("PROVIDER_INVALID_OUTPUT");
    return;
  }
  const auto token = reply.value("token").toString();
  if (!m_pending.remove(token) || token != m_current)
    return;
  m_current.clear();
  m_action.clear();
  m_watchdog.stop();
  m_reply = reply;
  clearPreview();
  if (reply.value("ok").toBool() && reply.contains("preview")) {
    const auto preview = reply.value("preview").toObject();
    const auto digest = preview.value("proposalDigest").toString();
    if (digest.size() != 64 || !preview.value("requiresApproval").toBool()) {
      fail("PROVIDER_INVALID_OUTPUT");
      return;
    }
    m_digest = digest;
    auto canonical = preview;
    canonical.remove("untrustedExplanation");
    m_preview->setPlainText(
        QString::fromUtf8(QJsonDocument(canonical).toJson()) +
        "\nNormalization profile: principal in GBP; annual rate in percent; term in months.\n"
        "Untrusted provider explanation:\n" +
        preview.value("untrustedExplanation").toString());
    const auto expires = preview.value("expiresInSeconds").toInt();
    if (expires <= 0 || expires > 300) {
      clearPreview();
      fail("APPROVAL_EXPIRED");
      return;
    }
    m_deadline.start(expires * 1000);
    m_status->setText(
        "Review the exact pin, inputs and effects. Approval is required.");
  } else {
    m_status->setText(reply.value("code").toString());
    if (reply.value("topologyCommitted").toBool()) {
      if (reply.value("topologyCheckpointPersisted").toBool()) {
        if (!reply.value("initializationSucceeded").toBool() ||
            !reply.value("checkpointPersisted").toBool())
          m_status->setText(
              "Tool added; initialization checkpoint was not confirmed. "
              "Edit the installed form to initialize or recalculate it.");
        m_shell.bridge().selectRoom(reply.value("roomId").toString());
      } else {
        m_status->setText("Topology persistence was not confirmed. Inspect the "
                          "refreshed canvas and storage diagnostics before "
                          "previewing the request again.");
        m_shell.bridge().selectRoom(m_shell.bridge().selectedRoom());
      }
    } else if (reply.value("refreshDesktop").toBool()) {
      m_shell.bridge().selectRoom(m_shell.bridge().selectedRoom());
    }
  }
  refresh();
  emit replyReceived();
}
