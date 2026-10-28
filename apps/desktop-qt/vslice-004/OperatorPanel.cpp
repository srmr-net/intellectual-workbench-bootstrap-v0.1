#include "OperatorPanel.h"
#include <QAccessible>
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QHeaderView>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QProcess>
#include <QProgressBar>
#include <QPushButton>
#include <QScrollArea>
#include <QSignalBlocker>
#include <QThread>
#include <QTreeWidget>
#include <QUuid>
#include <QVBoxLayout>
#include <QtEndian>
namespace {
QString statusText(const QString &code) {
  if (code == "MISSING_PARAMETER")
    return code +
           ": Supply all required values and units using the supported intent "
           "format, or use an installed form. No values were guessed.";
  if (code == "INVALID_UNIT")
    return code + ": Use the explicit units required by the admitted component "
                  "contract.";
  if (code == "AMBIGUOUS_INTENT")
    return code + ": More than one installed candidate matches. Choose an "
                  "exact component with the installed picker.";
  if (code == "STATE_CONFLICT" || code == "APPROVAL_EXPIRED")
    return code + ": The workspace or approval changed. Preview the request "
                  "again before confirming.";
  if (code == "PIN_UNAVAILABLE" || code == "CATALOG_CHANGED")
    return code + ": The exact proposed package is unavailable or changed. "
                  "Inspect installed components and preview again.";
  if (code == "PROVIDER_UNAVAILABLE")
    return code + ": The configured provider is disabled or unavailable. "
                  "Existing forms and saved canvases still work offline.";
  if (code == "AUTH_FAILED" || code == "NOT_CONFIGURED")
    return code +
           ": Choose a readable owner-only private credential source, then "
           "review and consent again. Saved forms remain available offline.";
  if (code == "RATE_LIMITED")
    return code + ": Provider capacity or quota is exhausted. Wait or check "
                  "the account before a new separately consented request; no "
                  "automatic retry.";
  if (code == "TIMEOUT" || code == "PROVIDER_TIMEOUT" ||
      code == "UNAVAILABLE" || code == "NETWORK_ERROR")
    return code +
           ": The request did not complete. Check connectivity, or select "
           "Offline. A new remote attempt requires another review and consent.";
  if (code == "REFUSED" || code == "TRUNCATED" || code == "SCHEMA_MISMATCH" ||
      code == "PROVIDER_ERROR" || code == "MALFORMED_RESPONSE")
    return code + ": No usable suggestion was accepted. Refine the explicit "
                  "request or use Offline; the existing room is unchanged.";
  if (code == "DISCLOSURE_EXPIRED" || code == "DISCLOSURE_REQUIRED")
    return code + ": Review the current outbound data again and give fresh "
                  "inference consent.";
  return code;
}
void showSection(QWidget *widget) {
  // Scroll only; preserve keyboard focus and never activate a consent control.
  QTimer::singleShot(0, widget, [widget] {
    // Visibility changes post a layout request; scroll after that layout too.
    QTimer::singleShot(0, widget, [widget] {
      for (auto *parent = widget->parentWidget(); parent;
           parent = parent->parentWidget())
        if (auto *scroll = qobject_cast<QScrollArea *>(parent))
          scroll->ensureWidgetVisible(widget, 16, 24);
    });
  });
}
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
  m_status = nullptr;
  m_proposalRows = nullptr;
  m_outbound = nullptr;
  m_remoteConsent = nullptr;
  setObjectName("operatorPanel");
  setAccessibleName("Request an installed tool");
  m_prefix = "op-" + QUuid::createUuid().toString(QUuid::Id128) + "-";
  auto *layout = new QVBoxLayout(this);
  auto *heading = new QLabel("Request an installed tool", this);
  heading->setTextFormat(Qt::PlainText);
  layout->addWidget(heading);
  m_providerChoice = new QComboBox(this);
  m_providerChoice->setObjectName("providerChoice");
  m_providerChoice->setAccessibleName("Suggestion provider");
  m_providerChoice->addItem("Offline deterministic", "offline-fixture");
  m_providerChoice->addItem("OpenRouter (remote)", "openrouter-chat");
  layout->addWidget(m_providerChoice);
  m_providerInfo = new QLabel("Remote inference is off by default.", this);
  m_providerInfo->setObjectName("providerInfo");
  m_providerInfo->setAccessibleName("Provider destination and limits");
  m_providerInfo->setTextFormat(Qt::PlainText);
  m_providerInfo->setWordWrap(true);
  layout->addWidget(m_providerInfo);
  m_remoteConsent = new QCheckBox("Consent to next remote request", this);
  m_remoteConsent->setObjectName("remoteConsent");
  m_remoteConsent->setAccessibleName(
      "Consent to remote request and catalog transfer");
  m_sourcePath = new QLineEdit(this);
  m_sourcePath->setObjectName("credentialPath");
  m_sourcePath->setAccessibleName(
      "Private credential file path, not the API key");
  m_sourcePath->setPlaceholderText(
      "Absolute path to an existing owner-only credential file");
  m_sourcePath->setMaxLength(4096);
  layout->addWidget(m_sourcePath);
  m_sourceApply =
      button("Use private credential file", "credentialApply", this);
  layout->addWidget(m_sourceApply);
  connect(m_sourceApply, &QPushButton::clicked, this, [this] {
    const auto path = m_sourcePath->text();
    m_sourcePath->clear();
    clearPreview();
    clearDisclosure();
    send({{"type", "credential"}, {"path", path}}, false, true);
  });
  auto *dataHeading =
      new QLabel("1. Review outbound data and consent to inference", this);
  dataHeading->setTextFormat(Qt::PlainText);
  dataHeading->setWordWrap(true);
  layout->addWidget(dataHeading);
  m_outbound = new QPlainTextEdit(this);
  m_outbound->setObjectName("outboundData");
  m_outbound->setAccessibleName("Data to send to OpenRouter");
  m_outbound->setReadOnly(true);
  m_outbound->setMaximumHeight(160);
  layout->addWidget(m_outbound);
  layout->addWidget(m_remoteConsent);
  m_send = button("Send reviewed data to OpenRouter", "operatorSend", this);
  layout->addWidget(m_send);
  connect(m_send, &QPushButton::clicked, this, [this] {
    if (!m_remoteConsent->isChecked() || m_reviewId.isEmpty() || busy())
      return;
    const auto id = m_reviewId, digest = m_reviewDigest;
    clearPreview();
    clearDisclosure();
    send({{"type", "send"}, {"reviewId", id}, {"reviewDigest", digest}}, false,
         true);
  });
  auto *disclosure = new QLabel(
      "Single-use consent: only the next explicit request and admitted catalog "
      "are sent; no documents, paths or history. This does not approve a plan. "
      "The selected provider's data policy is shown below.",
      this);
  disclosure->setObjectName("providerDisclosure");
  disclosure->setTextFormat(Qt::PlainText);
  disclosure->setWordWrap(true);
  disclosure->setAccessibleName("Remote inference disclosure");
  layout->addWidget(disclosure);
  connect(m_providerChoice, qOverload<int>(&QComboBox::currentIndexChanged),
          this, [this] {
            QSignalBlocker block(m_remoteConsent);
            m_remoteConsent->setChecked(false);
            configureProvider();
          });
  connect(m_remoteConsent, &QCheckBox::toggled, this, &OperatorPanel::refresh);
  m_intent = new QPlainTextEdit(this);
  m_intent->setObjectName("operatorIntent");
  m_intent->setAccessibleName("Tool request");
  m_intent->setPlaceholderText("Describe one tool and the values to use.");
  m_intent->setMaximumHeight(60);
  layout->addWidget(m_intent);
  m_prepare = button("Review request / outbound data", "operatorPrepare", this);
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
  auto *approvalHeading = new QLabel(
      "2. Review the host proposal and separately approve execution", this);
  approvalHeading->setTextFormat(Qt::PlainText);
  approvalHeading->setWordWrap(true);
  layout->addWidget(approvalHeading);
  m_proposalRows = new QTreeWidget(this);
  m_proposalRows->setObjectName("proposalDetails");
  m_proposalRows->setAccessibleName("Validated proposal details");
  m_proposalRows->setColumnCount(2);
  m_proposalRows->setHeaderLabels({"Host-validated field", "Value"});
  m_proposalRows->setMaximumHeight(230);
  m_proposalRows->setRootIsDecorated(false);
  m_proposalRows->header()->setStretchLastSection(true);
  layout->addWidget(m_proposalRows);

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
  m_progress = new QProgressBar(this);
  m_progress->setObjectName("requestProgress");
  m_progress->setAccessibleName("Remote request progress");
  m_progress->setRange(0, 0);
  m_progress->setVisible(false);
  layout->addWidget(m_progress);
  m_remoteConsent->setStyleSheet(
      "QCheckBox { border: 2px solid palette(highlight); padding: 5px; }");
  m_consent->setStyleSheet(
      "QCheckBox { border: 2px solid palette(text); padding: 5px; }");
  setTabOrder(m_providerChoice, m_sourcePath);
  setTabOrder(m_sourcePath, m_sourceApply);
  setTabOrder(m_sourceApply, m_intent);
  setTabOrder(m_intent, m_prepare);
  setTabOrder(m_prepare, m_outbound);
  setTabOrder(m_outbound, m_remoteConsent);
  setTabOrder(m_remoteConsent, m_send);
  setTabOrder(m_send, m_consent);
  setTabOrder(m_consent, m_approve);
  setTabOrder(m_approve, m_decline);
  setTabOrder(m_decline, m_cancel);
  m_progressTimer.setInterval(1000);
  connect(&m_progressTimer, &QTimer::timeout, this, [this] {
    if (m_action == "send" || m_action == "prepare")
      publishStatus(
          "REQUESTING: " + QString::number(m_progressClock.elapsed() / 1000) +
          " seconds elapsed. Cancel is available; provider work may continue "
          "after local cancellation.");
  });

  auto *context = qobject_cast<QVBoxLayout *>(parentWidget()->layout());
  auto *scroll = new QScrollArea(parentWidget());
  scroll->setWidgetResizable(true);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  scroll->setWidget(this);
  context->insertWidget(context->count() - 1, scroll, 1);
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
    clearDisclosure();
    m_submitted->setPlainText(utterance);
    if (m_providerChoice->currentIndex() == 1)
      send({{"type", "review"}, {"utterance", utterance}}, false, true);
    else
      send({{"type", "prepare"}, {"utterance", utterance}});
  });
  connect(m_intent, &QPlainTextEdit::textChanged, this, [this] {
    const bool revoke = !m_reviewId.isEmpty() || !m_digest.isEmpty();
    clearPreview();
    clearDisclosure();
    if (revoke && !busy())
      send({{"type", "withdraw"}}, false, true);
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
  connect(&shell.bridge(), &DesktopBridge::stateChanged, this, [this] {
    if (m_shell.bridge().switching() && !m_selectingApproved) {
      const bool active = m_action == "send" || m_action == "prepare";
      const bool reviewed = !m_reviewId.isEmpty() || !m_digest.isEmpty();
      clearPreview();
      clearDisclosure();
      if (active && m_pending.size() < 2)
        send({{"type", "cancel"}});
      else if (reviewed && !busy())
        send({{"type", "withdraw"}}, false, true);
    }
    refresh();
  });
  m_reviewDeadline.setSingleShot(true);
  connect(&m_reviewDeadline, &QTimer::timeout, this, [this] {
    clearDisclosure();
    send({{"type", "withdraw"}}, false, true);
    publishStatus("Outbound review expired. Review again before sending.");
    refresh();
  });
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
  for (auto *editor : findChildren<QPlainTextEdit *>())
    editor->setTabChangesFocus(true);
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
  m_preview->clear();
  m_consent->setChecked(false);
  if (m_proposalRows)
    m_proposalRows->clear();
  refresh();
}
void OperatorPanel::refresh() {
  const auto state = m_shell.bridge().lastState();
  const bool healthy = !state.isEmpty() && state.value("recovery").isNull() &&
                       m_process && m_process->state() == QProcess::Running;
  const bool canConfigure =
      healthy && m_action != "approve" && m_pending.size() < 2;
  m_providerChoice->setEnabled(canConfigure);
  m_remoteConsent->setEnabled(
      canConfigure && m_providerChoice->currentIndex() == 1 && !busy() &&
      m_digest.isEmpty() && !m_reviewId.isEmpty());
  m_sourcePath->setEnabled(canConfigure && !busy());
  m_sourceApply->setEnabled(canConfigure && !busy());
  m_send->setEnabled(
      healthy && !busy() && !m_reviewId.isEmpty() &&
      m_remoteConsent->isChecked() &&
      m_providerState.value("source").toObject().value("available").toBool());
  const bool requesting =
      busy() && (m_action == "send" || m_action == "prepare");
  m_progress->setVisible(requesting);
  if (!requesting)
    m_progressTimer.stop();
  m_prepare->setEnabled(healthy && !busy());
  m_intent->setEnabled(healthy && !busy());
  m_consent->setEnabled(!m_digest.isEmpty() && !busy());
  m_approve->setEnabled(!m_digest.isEmpty() && m_consent->isChecked() &&
                        !busy());
  m_decline->setEnabled(!m_digest.isEmpty() && !busy());
  m_cancel->setEnabled(busy() &&
                       (m_action == "prepare" || m_action == "send") &&
                       m_pending.size() < 2);
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
  publishStatus(statusText(code));
  refresh();
}
void OperatorPanel::send(const QJsonObject &command, bool provider, bool ux) {
  if (!m_process || m_process->state() != QProcess::Running ||
      m_pending.size() >= 2) {
    fail("QUEUE_FULL");
    return;
  }
  const auto token = m_prefix + QString::number(++m_token);
  const auto bytes =
      QJsonDocument(
          QJsonObject{{"protocol",
                       ux ? "ux.1" : (provider ? "provider.1" : "operator.1")},
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
  if (m_action == "send" || m_action == "prepare") {
    m_progressClock.restart();
    m_progressTimer.start();
    showSection(m_progress);
    publishStatus("REQUESTING: host validation and bounded provider request. "
                  "Cancel is available.");
  } else
    publishStatus("Waiting for host " + m_action + "…");
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
  if (reply.value("protocol") == "ux.1" && reply.value("kind") == "reply") {
    const auto token = reply.value("token").toString();
    if (!m_pending.remove(token) || token != m_current) {
      refresh();
      return;
    }
    m_current.clear();
    m_action.clear();
    m_watchdog.stop();
    m_uxReply = reply;
    if (reply.contains("providerState"))
      updateProvider(reply.value("providerState").toObject());
    clearDisclosure();
    clearPreview();
    if (reply.value("ok").toBool() && reply.contains("disclosure")) {
      m_review = reply.value("disclosure").toObject();
      m_reviewId = m_review.value("reviewId").toString();
      m_reviewDigest = m_review.value("reviewDigest").toString();
      m_reviewRoom = m_shell.bridge().selectedRoom();
      m_outbound->setPlainText(
          "REVIEW BEFORE SENDING — no inference has occurred.\n" +
          m_review.value("selectorPolicy").toString() + "\n" +
          m_review.value("providerDataUse").toString() + "\n" +
          m_review.value("exclusions").toString() + "\n" +
          QString::fromUtf8(
              QJsonDocument(m_review.value("outboundData").toObject())
                  .toJson()) +
          "\nFixed host instructions/schema (selector placeholders explained "
          "above):\n" +
          m_review.value("fixedHostContext").toString() +
          "\nHost request settings:\n" +
          QString::fromUtf8(
              QJsonDocument(m_review.value("requestSettings").toObject())
                  .toJson()));
      m_reviewDeadline.start(120000);
      showSection(m_remoteConsent);
      publishStatus("DISCLOSURE_READY: Review the exact request, catalog "
                    "fields and destination. Inference consent is separate "
                    "from execution approval.");
    } else if (reply.value("ok").toBool())
      publishStatus("READY: source/configuration updated; authentication is "
                    "checked only when sending.");
    else
      fail(reply.value("code").toString());
    refresh();
    emit replyReceived();
    return;
  }
  if (reply.value("protocol") == "provider.1" &&
      reply.value("kind") == "reply") {
    const auto token = reply.value("token").toString();
    if (!m_pending.remove(token) || token != m_current) {
      refresh();
      return;
    }
    m_current.clear();
    m_action.clear();
    m_watchdog.stop();
    clearPreview();
    clearDisclosure();
    updateProvider(reply);
    publishStatus(reply.value("code").toString());
    refresh();
    emit replyReceived();
    return;
  }
  if (reply.value("protocol") != "operator.1" ||
      reply.value("kind") != "reply") {
    clearPreview();
    fail("PROVIDER_INVALID_OUTPUT");
    return;
  }
  const auto token = reply.value("token").toString();
  if (!m_pending.remove(token) || token != m_current) {
    refresh();
    return;
  }
  m_current.clear();
  m_action.clear();
  m_watchdog.stop();
  m_reply = reply;
  if (reply.contains("providerState"))
    updateProvider(reply.value("providerState").toObject());
  clearPreview();
  if (reply.value("ok").toBool() && reply.contains("preview")) {
    const auto preview = reply.value("preview").toObject();
    const auto digest = preview.value("proposalDigest").toString();
    if (digest.size() != 64 || !preview.value("requiresApproval").toBool()) {
      fail("PROVIDER_INVALID_OUTPUT");
      return;
    }
    m_digest = digest;
    showPreview(reply);
    const auto expires = preview.value("expiresInSeconds").toInt();
    if (expires <= 0 || expires > 300) {
      clearPreview();
      fail("APPROVAL_EXPIRED");
      return;
    }
    m_deadline.start(expires * 1000);
    showSection(m_consent);
    publishStatus(
        "Review the exact pin, inputs and effects. Approval is required.");
  } else {
    publishStatus(statusText(reply.value("code").toString()));
    if (reply.value("topologyCommitted").toBool()) {
      if (reply.value("topologyCheckpointPersisted").toBool()) {
        if (!reply.value("initializationSucceeded").toBool())
          publishStatus(
              "Tool topology saved; initial typed calculation failed. No new "
              "output checkpoint was saved. Edit the installed form to "
              "initialize or recalculate it; do not resubmit topology.");
        else if (!reply.value("checkpointPersisted").toBool())
          publishStatus(
              "Tool topology saved and calculation validated, but its output "
              "checkpoint was not saved. Edit the installed form to "
              "recalculate; inspect storage diagnostics before closing.");
        else
          publishStatus("Tool topology saved; typed initialization succeeded "
                        "and validated output checkpoint saved. Form edits run "
                        "locally without further inference.");
        m_selectingApproved = true;
        m_shell.bridge().selectRoom(reply.value("roomId").toString());
        m_selectingApproved = false;
      } else {
        publishStatus("Topology persistence was not confirmed. Inspect the "
                      "refreshed canvas and storage diagnostics before "
                      "previewing the request again.");
        m_shell.bridge().selectRoom(m_shell.bridge().selectedRoom());
      }
    } else if (reply.value("refreshDesktop").toBool()) {
      m_shell.bridge().selectRoom(m_shell.bridge().selectedRoom());
    }
  }
  if (!reply.value("ok").toBool() && reply.contains("providerAudit")) {
    const auto code = reply.value("providerAudit")
                          .toObject()
                          .value("observation")
                          .toObject()
                          .value("status")
                          .toString();
    if (!code.isEmpty() && code != "OK")
      fail(code);
  }
  refresh();
  emit replyReceived();
}

void OperatorPanel::configureProvider() {
  clearPreview();
  clearDisclosure();
  send({{"type", "configure"},
        {"provider", m_providerChoice->currentData().toString()},
        {"consent", false}},
       true);
}
void OperatorPanel::updateProvider(const QJsonObject &state) {
  m_providerState = state;
  const auto profile = state.value("profile").toObject();
  QSignalBlocker choice(m_providerChoice), consent(m_remoteConsent);
  const auto remote = state.value("remoteProvider").toString();
  m_providerChoice->setItemData(1, remote);
  m_providerChoice->setItemText(1, "OpenRouter (remote)");
  m_providerChoice->setCurrentIndex(profile.value("provider") == remote ? 1
                                                                        : 0);
  auto *disclosure = findChild<QLabel *>("providerDisclosure");
  disclosure->setText(
      "Single-use consent sends only the next explicit request and admitted "
      "catalog, not documents, paths or history. Plan approval is separate. " +
      QString(
          remote == "openrouter-chat"
              ? "OpenRouter routes to NVIDIA's free endpoint, which logs "
                "requests for security and product improvement. No provider "
                "JSON Schema mode; strict local validation remains mandatory."
              : "The selected provider's data policy is shown below. Strict "
                "provider schema plus mandatory local validation."));

  const auto budget = profile.value("budget").toObject();
  m_providerInfo->setText(
      state.value("remoteModel").toString() + "\n" +
      state.value("endpoint").toString() +
      "\nTransport: " + state.value("transport").toString() + "; attempts: " +
      state.value("remoteAttempts").toString() + "; max output tokens: " +
      QString::number(budget.value("maxOutputTokens").toInt()) +
      "; credential source: " +
      (state.value("source").toObject().value("available").toBool()
           ? "available (authentication checked at send)"
           : "unavailable — choose an owner-only private file or approved host "
             "environment source"));
}

void OperatorPanel::clearDisclosure() {
  m_reviewDeadline.stop();
  m_reviewId.clear();
  m_reviewDigest.clear();
  m_review = {};
  if (m_remoteConsent) {
    QSignalBlocker b(m_remoteConsent);
    m_remoteConsent->setChecked(false);
  }
  if (m_outbound)
    m_outbound->clear();
}
void OperatorPanel::publishStatus(const QString &text) {
  if (!m_status || m_status->text() == text)
    return;
  m_status->setText(text);
  m_status->setAccessibleDescription(text);
  QAccessibleEvent event(m_status, QAccessible::Alert);
  QAccessible::updateAccessibility(&event);
}
void OperatorPanel::showPreview(const QJsonObject &reply) {
  const auto p = reply.value("preview").toObject(),
             plan = p.value("plan").toObject();
  const auto instances = plan.value("instances").toArray();
  const auto component =
      instances.isEmpty()
          ? QJsonObject()
          : instances[0].toObject().value("component").toObject();
  QStringList summary;
  auto row = [&](const QString &name, const QString &value) {
    new QTreeWidgetItem(m_proposalRows, {name, value});
    summary << name + ": " + value;
  };
  row("Component", component.value("id").toString());
  row("Version", component.value("version").toString());
  row("Full artifact SHA-256", component.value("sha256").toString());
  row("Affected room", plan.value("roomId").toString());
  row("Expected room revision",
      QString::number(plan.value("expectedRoomRevision").toInt()));
  const auto fields =
      reply.value("presentation").toObject().value("fields").toArray();
  for (const auto &v : fields) {
    const auto f = v.toObject();
    const auto value = f.value("value");
    const auto text =
        value.isString() ? value.toString() : QString::number(value.toInt());
    row(f.value("label").toString() + " [" + f.value("field").toString() + "]",
        text + " " + f.value("unit").toString());
  }
  row("Capabilities",
      QString::fromUtf8(QJsonDocument(reply.value("presentation")
                                          .toObject()
                                          .value("capabilities")
                                          .toArray())
                            .toJson(QJsonDocument::Compact)));
  row("Requested permissions",
      QString::fromUtf8(
          QJsonDocument(plan.value("requestedPermissions").toArray())
              .toJson(QJsonDocument::Compact)));
  row("Planned effect", p.value("effect").toString());
  row("On confirmation",
      "Commit topology first; typed initialization and validated output "
      "checkpoint are separate operations.");
  row("Proposal digest", p.value("proposalDigest").toString());
  row("Expiry",
      QString::number(p.value("expiresInSeconds").toInt()) + " seconds");
  row("Provider",
      m_providerState.value("profile").toObject().value("provider").toString());
  row("Untrusted provider explanation",
      p.value("untrustedExplanation").toString());
  m_preview->setPlainText(summary.join("\n"));
}
