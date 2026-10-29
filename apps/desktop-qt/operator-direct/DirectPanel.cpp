#include "DirectPanel.h"
#include <QAccessible>
#include <QCheckBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QGroupBox>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QScrollArea>
#include <QUuid>
#include <QVBoxLayout>
#include <QtEndian>
namespace {
QLabel *label(QString text, QWidget *parent) {
  auto *l = new QLabel(text, parent);
  l->setTextFormat(Qt::PlainText);
  l->setWordWrap(true);
  return l;
}
QPushButton *button(QString text, QString name, QWidget *parent) {
  auto *b = new QPushButton(text, parent);
  b->setObjectName(name);
  b->setAccessibleName(text);
  return b;
}
QString textValue(QJsonValue v) {
  return v.isString()
             ? v.toString()
             : QString::fromUtf8(
                   QJsonDocument(QJsonArray{v}).toJson(QJsonDocument::Compact))
                   .mid(1)
                   .chopped(1);
}
} // namespace
DirectPanel::DirectPanel(DesktopShell &shell, bool live)
    : QWidget(shell.findChild<QWidget *>("activityContext")), m_shell(shell),
      m_process(shell.bridge().findChild<QProcess *>()), m_remote(live) {
  m_prefix =
      "direct-" + QUuid::createUuid().toString(QUuid::Id128).left(12) + "-";
  m_file = qEnvironmentVariable("WB_LLM_SECRET_FILE");
  if (auto *identity = shell.findChild<QLabel *>("selectedIdentity"))
    identity->hide();
  auto *layout = new QVBoxLayout(this);
  layout->addWidget(label("Operator", this));
  m_providerLabel = label("Opening provider…", this);
  layout->addWidget(m_providerLabel);
  m_settings = button("Settings", "directSettings", this);
  layout->addWidget(m_settings);
  connect(m_settings, &QPushButton::clicked, this, &DirectPanel::settings);
  m_request = new QPlainTextEdit(this);
  m_request->setObjectName("directRequest");
  m_request->setAccessibleName("What would you like to do?");
  m_request->setPlaceholderText(
      "What would you like to do? Describe a tool and its inputs.");
  m_request->setTabChangesFocus(true);
  m_request->setMaximumHeight(110);
  layout->addWidget(m_request);
  m_ask = button("Ask Operator", "directAsk", this);
  layout->addWidget(m_ask);
  m_cancel = button("Cancel request", "directCancel", this);
  layout->addWidget(m_cancel);
  m_status =
      label("Ask for a tool from the installed component catalog.", this);
  m_status->setObjectName("directStatus");
  m_status->setAccessibleName("Operator status");
  layout->addWidget(m_status);
  m_card = new QGroupBox("Suggested tool", this);
  auto *card = new QVBoxLayout(m_card);
  m_summary = label("", m_card);
  card->addWidget(m_summary);
  card->addWidget(
      label("Start adds this tool, then supplies these inputs. Calculations "
            "run locally after starting.",
            m_card));
  m_start = button("Start this tool", "directStart", m_card);
  m_reject = button("Dismiss", "directDismiss", m_card);
  card->addWidget(m_start);
  card->addWidget(m_reject);
  layout->addWidget(m_card);
  m_card->hide();
  auto *detailButton = button("Technical details", "directDetails", this);
  layout->addWidget(detailButton);
  m_details = new QPlainTextEdit(this);
  m_details->setReadOnly(true);
  m_details->setMaximumHeight(160);
  m_details->hide();
  layout->addWidget(m_details);
  connect(detailButton, &QPushButton::clicked, this,
          [this] { m_details->setVisible(!m_details->isVisible()); });
  layout->addStretch();
  auto *context = qobject_cast<QVBoxLayout *>(parentWidget()->layout());
  auto *scroll = new QScrollArea(parentWidget());
  scroll->setWidgetResizable(true);
  scroll->setWidget(this);
  context->insertWidget(context->count() - 1, scroll, 1);
  QObject::disconnect(m_process, SIGNAL(readyReadStandardError()),
                      &shell.bridge(), nullptr);
  connect(m_process, &QProcess::readyReadStandardError, this,
          &DirectPanel::read);
  connect(m_process, &QProcess::started, this, &DirectPanel::configure);
  if (m_process->state() == QProcess::Running)
    QTimer::singleShot(0, this, &DirectPanel::configure);
  connect(m_ask, &QPushButton::clicked, this, [this] {
    if (busy())
      return;
    clearProposal();
    const auto utterance = m_request->toPlainText();
    if (utterance.trimmed().isEmpty()) {
      status("Describe the tool you need first.");
      return;
    }
    send(m_remote ? "ux.1" : "operator.1",
         {{"type", m_remote ? "review" : "prepare"}, {"utterance", utterance}});
    status(m_remote ? "Asking OpenRouter…" : "Looking for an installed tool…");
  });
  connect(m_start, &QPushButton::clicked, this, [this] {
    if (!busy() && !m_digest.isEmpty()) {
      const auto digest = m_digest;
      clearProposal();
      send("operator.1", {{"type", "approve"}, {"digest", digest}});
      status("Starting the installed tool…");
    }
  });
  connect(m_reject, &QPushButton::clicked, this, [this] {
    clearProposal();
    send("operator.1", {{"type", "decline"}});
  });
  connect(m_cancel, &QPushButton::clicked, this, [this] {
    clearProposal();
    send("operator.1", {{"type", "cancel"}});
  });
  connect(m_request, &QPlainTextEdit::textChanged, this, [this] {
    if (!m_digest.isEmpty()) {
      clearProposal();
      status("Request changed. Ask Operator again.");
    }
  });
  connect(&shell.bridge(), &DesktopBridge::stateChanged, this, [this] {
    if (m_shell.bridge().switching() && m_phase != "approve") {
      clearProposal();
      if (busy() &&
          (m_phase == "review" || m_phase == "send" || m_phase == "prepare"))
        send("operator.1", {{"type", "cancel"}});
    }
    refresh();
  });
  m_expiry.setSingleShot(true);
  connect(&m_expiry, &QTimer::timeout, this, [this] {
    clearProposal();
    status("This suggestion expired. Ask Operator again.");
  });
  m_watchdog.setSingleShot(true);
  connect(&m_watchdog, &QTimer::timeout, this, [this] {
    if (m_phase == "approve") {
      status("Start is still pending. Inspect the canvas before retrying.");
      return;
    }
    send("operator.1", {{"type", "cancel"}});
    status("The request timed out. Try again or use Offline.");
  });
  refresh();
}
void DirectPanel::status(QString text) {
  m_status->setText(text);
  m_status->setAccessibleDescription(text);
  QAccessibleEvent event(m_status, QAccessible::Alert);
  QAccessible::updateAccessibility(&event);
}
void DirectPanel::clearProposal() {
  m_digest.clear();
  m_preview = {};
  m_expiry.stop();
  m_card->hide();
  refresh();
}
void DirectPanel::refresh() {
  const auto state = m_shell.bridge().lastState();
  const bool healthy = !state.isEmpty() && state.value("recovery").isNull() &&
                       m_process->state() == QProcess::Running;
  m_ask->setEnabled(healthy && m_configured && !busy());
  m_request->setEnabled(!busy());
  m_settings->setEnabled(!busy());
  m_start->setEnabled(healthy && !busy() && !m_digest.isEmpty());
  m_reject->setEnabled(!busy());
  m_cancel->setVisible(busy() && (m_phase == "review" || m_phase == "send" ||
                                  m_phase == "prepare"));
}
void DirectPanel::configure() {
  clearProposal();
  send("provider.1",
       {{"type", "configure"},
        {"provider", m_remote ? "openrouter-chat" : "offline-fixture"},
        {"consent", false}});
}
void DirectPanel::settings() {
  QDialog dialog(this);
  dialog.setWindowTitle("Operator settings");
  auto *v = new QVBoxLayout(&dialog);
  auto *enabled =
      new QCheckBox("Use OpenRouter for my requests this session", &dialog);
  enabled->setChecked(m_remote);
  v->addWidget(enabled);
  v->addWidget(
      label("When enabled, Ask sends your typed request and a limited "
            "installed-tool catalog to OpenRouter/NVIDIA. NVIDIA's free "
            "endpoint logs requests. No documents or chat history are sent "
            "automatically. Each call is bounded; the model cannot start "
            "tools. You start a validated suggestion separately.",
            &dialog));
  v->addWidget(label("Private credential file path (not the key)", &dialog));
  auto *source = new QLineEdit(m_file, &dialog);
  source->setMaxLength(4096);
  v->addWidget(source);
  auto *buttons = new QDialogButtonBox(
      QDialogButtonBox::Save | QDialogButtonBox::Cancel, &dialog);
  v->addWidget(buttons);
  connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
  connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
  if (dialog.exec() != QDialog::Accepted)
    return;
  m_remote = enabled->isChecked();
  if (source->text() != m_file) {
    m_file = source->text();
    m_sourceThenConfigure = true;
    send("ux.1", {{"type", "credential"},
                  {"path", m_file.isEmpty() ? QJsonValue(QJsonValue::Null)
                                            : QJsonValue(m_file)}});
  } else
    configure();
}
void DirectPanel::send(QString protocol, QJsonObject command) {
  if (m_process->state() != QProcess::Running) {
    status("The local host is unavailable. Restart Workbench.");
    return;
  }
  const auto token = m_prefix + QString::number(++m_sequence);
  auto data = QJsonDocument(QJsonObject{{"protocol", protocol},
                                        {"token", token},
                                        {"command", command}})
                  .toJson(QJsonDocument::Compact);
  if (data.size() > 20 * 1024) {
    status("Request too large.");
    return;
  }
  QByteArray frame(4, '\0');
  qToBigEndian<quint32>(data.size(), frame.data());
  frame += data;
  m_token = token;
  m_phase = command.value("type").toString();
  if (m_process->write(frame) != frame.size()) {
    m_token.clear();
    status("Unable to contact the local host.");
  }
  m_watchdog.start(60000);
  refresh();
}
void DirectPanel::read() {
  const auto saved = m_process->readChannel();
  m_process->setReadChannel(QProcess::StandardError);
  m_buffer += m_process->readAll();
  if (m_buffer.size() > 65540) {
    m_buffer.clear();
    status("Invalid host response.");
    m_process->kill();
    return;
  }
  while (m_buffer.size() >= 4) {
    const auto n = qFromBigEndian<quint32>(m_buffer.constData());
    if (!n || n > 65536) {
      m_process->kill();
      status("Invalid host frame.");
      return;
    }
    if (m_buffer.size() < int(n) + 4)
      break;
    const auto bytes = m_buffer.mid(4, n);
    m_buffer.remove(0, n + 4);
    QJsonParseError error;
    auto doc = QJsonDocument::fromJson(bytes, &error);
    if (error.error != QJsonParseError::NoError || !doc.isObject()) {
      status("Invalid host response.");
      continue;
    }
    reply(doc.object());
  }
  m_process->setReadChannel(saved);
}
void DirectPanel::reply(const QJsonObject &r) {
  if (r.value("kind") != "reply" || r.value("token").toString() != m_token)
    return;
  const auto phase = m_phase;
  m_token.clear();
  m_phase.clear();
  m_watchdog.stop();
  if (r.contains("providerState"))
    m_provider = r.value("providerState").toObject();
  if (r.value("protocol") == "provider.1") {
    m_provider = r;
    m_configured = r.value("ok").toBool();
    m_providerLabel->setText(m_remote
                                 ? "OpenRouter · NVIDIA free model"
                                 : "Offline · deterministic local provider");
    status(m_configured
               ? "Ready. Describe the tool you need."
               : "Provider unavailable. Check Settings or select Offline.");
    refresh();
    return;
  }
  if (!r.value("ok").toBool()) {
    const auto code = r.value("providerAudit")
                          .toObject()
                          .value("observation")
                          .toObject()
                          .value("status")
                          .toString(r.value("code").toString());
    status(code + ": Request not completed. Check Settings, refine the "
                  "request, or try Offline.");
    if (r.value("topologyCommitted").toBool()) {
      status("Tool added, but initialization or saving failed. Inspect the "
             "canvas and edit its form; do not start a duplicate.");
      if (r.value("topologyCheckpointPersisted").toBool())
        m_shell.bridge().selectRoom(r.value("roomId").toString());
    }
    refresh();
    return;
  }
  if (phase == "credential" && m_sourceThenConfigure) {
    m_sourceThenConfigure = false;
    configure();
    return;
  }
  if (phase == "review") {
    // The explicit Ask gesture plus session provider opt-in authorizes this
    // call. The unchanged native host still binds/spends its one-use
    // actual-body guard.
    if (!m_remote) {
      refresh();
      return;
    }
    const auto d = r.value("disclosure").toObject();
    m_details->setPlainText(QString::fromUtf8(QJsonDocument(d).toJson()));
    send("ux.1", {{"type", "send"},
                  {"reviewId", d.value("reviewId")},
                  {"reviewDigest", d.value("reviewDigest")}});
    return;
  }
  if (r.contains("preview")) {
    m_preview = r.value("preview").toObject();
    m_digest = m_preview.value("proposalDigest").toString();
    const auto plan = m_preview.value("plan").toObject();
    const auto instances = plan.value("instances").toArray();
    if (m_digest.size() != 64 ||
        !m_preview.value("requiresApproval").toBool() ||
        instances.size() != 1) {
      clearProposal();
      status("Invalid suggestion.");
      return;
    }
    const auto component =
        instances[0].toObject().value("component").toObject();
    QStringList lines{component.value("id").toString(),
                      "Version " + component.value("version").toString()};
    for (const auto &v :
         r.value("presentation").toObject().value("fields").toArray()) {
      const auto f = v.toObject();
      lines << f.value("label").toString() + ": " +
                   textValue(f.value("value")) + " " +
                   f.value("unit").toString();
    }
    const auto permissions = plan.value("requestedPermissions").toArray();
    lines << (permissions.isEmpty()
                  ? "Permissions: none requested"
                  : "Permissions: " +
                        QString::fromUtf8(QJsonDocument(permissions)
                                              .toJson(QJsonDocument::Compact)));
    m_summary->setText(lines.join("\n"));
    m_details->setPlainText(QString::fromUtf8(QJsonDocument(r).toJson()));
    m_card->show();
    QTimer::singleShot(0, this, [this] {
      for (auto *p = parentWidget(); p; p = p->parentWidget())
        if (auto *scroll = qobject_cast<QScrollArea *>(p))
          scroll->ensureWidgetVisible(m_start);
    });
    const auto expiry = m_preview.value("expiresInSeconds").toInt();
    if (expiry <= 0 || expiry > 300) {
      clearProposal();
      status("Suggestion expired.");
      return;
    }
    m_expiry.start(expiry * 1000);
    status("Ready to start. Review the tool and inputs above.");
    refresh();
    return;
  }
  if (r.value("topologyCommitted").toBool()) {
    if (r.value("topologyCheckpointPersisted").toBool())
      m_shell.bridge().selectRoom(r.value("roomId").toString());
    status(r.value("initializationSucceeded").toBool() &&
                   r.value("checkpointPersisted").toBool()
               ? "Tool started. Edit its form; calculations now run locally "
                 "without the model."
               : "Tool added, but calculation or saving did not finish. Edit "
                 "the installed form to recover.");
  } else
    status(r.value("code") == "CANCELLED" ? "Request cancelled." : "Ready.");
  refresh();
}
