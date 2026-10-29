#include "DirectPanel.h"
#include "WorkspaceView.h"
#include <QAccessible>
#include <QAction>
#include <QApplication>
#include <QCheckBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QGroupBox>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QKeySequence>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPointer>
#include <QProcess>
#include <QProgressBar>
#include <QPushButton>
#include <QScrollArea>
#include <QTabWidget>
#include <QTextDocument>
#include <QUuid>
#include <QVBoxLayout>
#include <QtEndian>
namespace {
QLabel *label(QString text, QWidget *parent) {
  auto *l = new QLabel(text, parent);
  l->setTextFormat(Qt::PlainText);
  l->setWordWrap(true);
  l->setSizePolicy(QSizePolicy::Preferred, QSizePolicy::Maximum);
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
  auto *layout = new QVBoxLayout(this);
  m_providerLabel = label("Connecting…", this);
  m_providerLabel->setObjectName("operatorProviderStatus");
  layout->addWidget(m_providerLabel);
  m_settings = button("Settings", "directSettings", this);
  connect(m_settings, &QPushButton::clicked, this, &DirectPanel::settings);
  m_conversation = new QPlainTextEdit(this);
  m_conversation->setObjectName("operatorConversation");
  m_conversation->setAccessibleName("Operator conversation for this session");
  m_conversation->setReadOnly(true);
  m_conversation->setTabChangesFocus(true);
  m_conversation->document()->setMaximumBlockCount(100);
  m_conversation->setMinimumHeight(100);
  layout->addWidget(m_conversation, 1);
  m_request = new QPlainTextEdit(this);
  m_request->setObjectName("directRequest");
  m_request->setAccessibleName("Request to Operator");
  m_request->setPlaceholderText("What would you like to do?");
  m_request->setTabChangesFocus(true);
  m_request->setMaximumHeight(85);
  layout->addWidget(m_request);
  auto *actions = new QHBoxLayout;
  m_ask = button("Ask Operator", "directAsk", this);
  m_cancel = button("Cancel", "directCancel", this);
  actions->addWidget(m_ask);
  actions->addWidget(m_cancel);
  layout->addLayout(actions);
  m_progress = new QProgressBar(this);
  m_progress->setObjectName("operatorProgress");
  m_progress->setAccessibleName("Operator request in progress");
  m_progress->setRange(0, 0);
  m_progress->setTextVisible(false);
  m_progress->setFixedHeight(8);
  m_progress->hide();
  layout->addWidget(m_progress);
  m_status = label("Describe the tool you need and its inputs.", this);
  m_status->setObjectName("directStatus");
  m_status->setAccessibleName("Operator status");
  layout->addWidget(m_status);
  m_card = new QGroupBox("Suggested tool", this);
  m_card->setObjectName("operatorSuggestion");
  auto *card = new QVBoxLayout(m_card);
  m_summary = label("", m_card);
  m_summary->setObjectName("operatorSuggestionSummary");
  m_summary->setAccessibleName("Proposed tool and input parameters");
  card->addWidget(m_summary);
  card->addWidget(label("Start adds this tool and supplies these inputs. You "
                        "can then edit its form locally.",
                        m_card));
  auto *approval = new QHBoxLayout;
  m_start = button("Start this tool", "directStart", m_card);
  m_reject = button("Dismiss", "directDismiss", m_card);
  approval->addWidget(m_start);
  approval->addWidget(m_reject);
  auto *proposalDetails = button("Details", "proposalDetails", m_card);
  approval->addWidget(proposalDetails);
  card->addLayout(approval);
  layout->addWidget(m_card);
  m_card->hide();
  // Settings owns all component administration and technical information.
  m_dialog = new QDialog(&shell);
  m_dialog->setObjectName("workbenchSettings");
  m_dialog->setWindowTitle("Workbench Settings");
  m_dialog->resize(650, 600);
  connect(proposalDetails, &QPushButton::clicked, this, [this] {
    m_dialog->findChild<QTabWidget *>("settingsSections")->setCurrentIndex(2);
    settings();
  });
  auto *settingsLayout = new QVBoxLayout(m_dialog);
  auto *tabs = new QTabWidget(m_dialog);
  tabs->setObjectName("settingsSections");
  settingsLayout->addWidget(tabs);
  auto *provider = new QWidget(tabs);
  provider->setObjectName("providerSettings");
  auto *pv = new QVBoxLayout(provider);
  m_enabled =
      new QCheckBox("Enable OpenRouter for requests this session", provider);
  m_enabled->setObjectName("providerEnabled");
  m_enabled->setAccessibleName("Enable OpenRouter for this session");
  pv->addWidget(m_enabled);
  pv->addWidget(label(
      "Ask sends only your current typed request and a bounded installed-tool "
      "catalog. Conversation history and documents are not sent automatically. "
      "NVIDIA's free endpoint logs requests. The host validates suggestions; "
      "only your Start action can start a tool.",
      provider));
  pv->addWidget(label("Model — current supported provider profile", provider));
  m_model = new QLineEdit(provider);
  m_model->setObjectName("providerModel");
  m_model->setAccessibleName("Configured provider model");
  m_model->setReadOnly(true);
  pv->addWidget(m_model);
  pv->addWidget(label("Approved endpoint", provider));
  m_endpoint = new QLineEdit(provider);
  m_endpoint->setObjectName("providerEndpoint");
  m_endpoint->setAccessibleName("Approved HTTPS endpoint");
  m_endpoint->setReadOnly(true);
  pv->addWidget(m_endpoint);
  pv->addWidget(
      label("Private credential file path — never paste the key", provider));
  m_source = new QLineEdit(provider);
  m_source->setObjectName("providerCredentialSource");
  m_source->setAccessibleName(
      "Private credential file reference, not key contents");
  m_source->setMaxLength(4096);
  pv->addWidget(m_source);
  m_sourceStatus = label("", provider);
  m_sourceStatus->setObjectName("providerSourceStatus");
  pv->addWidget(m_sourceStatus);
  pv->addWidget(label("Model and endpoint are restricted to the existing "
                      "supported profile. Settings and consent last for this "
                      "process; secrets are never saved with workspaces.",
                      provider));
  pv->addStretch();
  tabs->addTab(provider, "Provider");
  auto *components = new QWidget(tabs);
  components->setObjectName("componentSettings");
  tabs->addTab(components, "Components");
  m_details = new QPlainTextEdit(tabs);
  m_details->setObjectName("operatorDiagnostics");
  m_details->setAccessibleName("Latest host validation details");
  m_details->setReadOnly(true);
  m_details->setTabChangesFocus(true);
  tabs->addTab(m_details, "Diagnostics");
  auto *buttons = new QDialogButtonBox(
      QDialogButtonBox::Save | QDialogButtonBox::Close, m_dialog);
  buttons->setObjectName("settingsButtons");
  settingsLayout->addWidget(buttons);
  connect(buttons, &QDialogButtonBox::accepted, m_dialog, &QDialog::accept);
  connect(buttons, &QDialogButtonBox::rejected, m_dialog, &QDialog::reject);
  auto *context = qobject_cast<QVBoxLayout *>(parentWidget()->layout());
  auto *scroll = new QScrollArea(parentWidget());
  scroll->setObjectName("operatorScroll");
  scroll->setWidgetResizable(true);
  scroll->setHorizontalScrollBarPolicy(Qt::ScrollBarAlwaysOff);
  scroll->setWidget(this);
  context->insertWidget(context->count() - 1, scroll, 1);
  m_workspace = new WorkspaceView(shell, components, this);
  auto *nav = qobject_cast<QVBoxLayout *>(
      shell.findChild<QWidget *>("leftPinnedTools")->layout());
  nav->insertWidget(
      nav->indexOf(shell.findChild<QPushButton *>("quitWorkbench")),
      m_settings);
  conversation("Operator",
               "Tell me which installed tool you need and the values to use. I "
               "will suggest it for you to review.");
  m_progressTimer.setInterval(1000);
  connect(&m_progressTimer, &QTimer::timeout, this, [this] {
    if (busy() &&
        (m_phase == "review" || m_phase == "send" || m_phase == "prepare"))
      status("Working on your request… " +
             QString::number(m_elapsed.elapsed() / 1000) +
             "s. You can cancel.");
  });
  auto *settingsAction = new QAction("Settings", &shell);
  settingsAction->setShortcut(QKeySequence("Ctrl+,"));
  settingsAction->setShortcutContext(Qt::ApplicationShortcut);
  shell.addAction(settingsAction);
  connect(settingsAction, &QAction::triggered, this, [this] {
    if (m_dialog->isVisible())
      m_dialog->activateWindow();
    else
      m_settings->click();
  });
  setTabOrder(shell.findChild<QListWidget *>("pinnedRooms"),
              shell.findChild<QListWidget *>("workspacePinnedTools"));
  setTabOrder(shell.findChild<QListWidget *>("workspacePinnedTools"),
              shell.findChild<QListWidget *>("workspaceRecentActivity"));
  setTabOrder(shell.findChild<QListWidget *>("workspaceRecentActivity"),
              shell.findChild<QPushButton *>("pinCanvas"));
  setTabOrder(shell.findChild<QPushButton *>("pinCanvas"), m_settings);
  setTabOrder(m_settings, shell.findChild<QPushButton *>("quitWorkbench"));
  setTabOrder(shell.findChild<QPushButton *>("quitWorkbench"), m_conversation);
  setTabOrder(m_conversation, m_request);
  setTabOrder(m_request, m_ask);
  setTabOrder(m_ask, m_cancel);
  setTabOrder(m_cancel, m_start);
  setTabOrder(m_start, m_reject);
  setTabOrder(m_reject, proposalDetails);
  setTabOrder(proposalDetails, shell.findChild<QWidget *>("semanticCanvas"));
  QObject::disconnect(m_process, SIGNAL(readyReadStandardError()),
                      &shell.bridge(), nullptr);
  connect(m_process, &QProcess::readyReadStandardError, this,
          &DirectPanel::read);
  connect(m_process, &QProcess::started, this, &DirectPanel::configure);
  if (m_process->state() == QProcess::Running)
    QTimer::singleShot(0, this, &DirectPanel::configure);
  connect(m_ask, &QPushButton::clicked, this, [this] {
    if (busy() || m_shell.bridge().busy() || m_shell.bridge().switching() ||
        m_shell.bridge().quitting())
      return;
    clearProposal();
    m_details->clear();
    const auto utterance = m_request->toPlainText();
    if (utterance.trimmed().isEmpty()) {
      status("Describe the tool you need first.");
      return;
    }
    if (utterance.toUtf8().size() > 4096) {
      status("Please shorten your request before asking again.");
      return;
    }
    conversation("You", utterance);
    m_elapsed.restart();
    m_progressTimer.start();
    send(m_remote ? "ux.1" : "operator.1",
         {{"type", m_remote ? "review" : "prepare"}, {"utterance", utterance}});
    status(m_remote ? "Asking OpenRouter…" : "Looking for an installed tool…");
  });
  connect(m_start, &QPushButton::clicked, this, [this] {
    if (!busy() && !m_shell.bridge().busy() && !m_shell.bridge().switching() &&
        !m_shell.bridge().quitting() && !m_digest.isEmpty()) {
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
  connect(&shell.bridge(), &DesktopBridge::busyChanged, this,
          &DirectPanel::refresh);
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
                       m_process->state() == QProcess::Running &&
                       !m_shell.bridge().quitting();
  const bool usable =
      !m_remote ||
      (m_provider.value("remoteConfigured").toBool() &&
       m_provider.value("source").toObject().value("available").toBool());
  m_ask->setEnabled(healthy && m_configured && usable && !busy() &&
                    !m_shell.bridge().busy() && !m_shell.bridge().switching());
  // Retain keyboard focus while waiting; disabling the editor transfers
  // focus into the native canvas and can submit an unrelated form edit.
  m_request->setReadOnly(busy());
  m_settings->setEnabled(m_process->state() == QProcess::Running);
  m_start->setEnabled(healthy && !busy() && !m_shell.bridge().busy() &&
                      !m_shell.bridge().switching() && !m_digest.isEmpty());
  m_reject->setEnabled(!busy());
  const bool requesting = busy() && (m_phase == "review" || m_phase == "send" ||
                                     m_phase == "prepare");
  m_progress->setVisible(requesting);
  if (!requesting)
    m_progressTimer.stop();
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
void DirectPanel::providerInformation() {
  m_model->setText(m_provider.value("remoteModel").toString());
  m_endpoint->setText(m_provider.value("endpoint").toString());
  const auto source = m_provider.value("source").toObject();
  m_sourceStatus->setText(source.value("available").toBool()
                              ? "Host credential source available. "
                                "Authentication is checked when sending."
                              : "No usable host credential source. Choose a "
                                "private owner-only file or use Offline.");
}
void DirectPanel::settings() {
  m_enabled->setChecked(m_remote);
  m_source->setText(m_file);
  providerInformation();
  m_enabled->setEnabled(!busy());
  m_source->setEnabled(!busy());
  auto *buttons = m_dialog->findChild<QDialogButtonBox *>("settingsButtons");
  buttons->button(QDialogButtonBox::Save)->setEnabled(!busy());
  QPointer<QWidget> prior = QApplication::focusWidget();
  const int result = m_dialog->exec();
  QTimer::singleShot(0, this, [this, prior] {
    if (prior && prior->isVisible() && prior->isEnabled()) {
      m_shell.activateWindow();
      prior->setFocus(Qt::OtherFocusReason);
    }
  });
  // A private reference is transient; never retain edit contents in the dialog.
  const auto file = m_source->text();
  const bool remote = m_enabled->isChecked();
  m_source->clear();
  if (result != QDialog::Accepted || busy())
    return;
  if (remote == m_remote && file == m_file)
    return;
  m_remote = remote;
  if (file != m_file) {
    m_file = file;
    m_sourceThenConfigure = true;
    send("ux.1", {{"type", "credential"},
                  {"path", m_file.isEmpty() ? QJsonValue(QJsonValue::Null)
                                            : QJsonValue(m_file)}});
  } else
    configure();
}
void DirectPanel::conversation(const QString &who, const QString &text) {
  m_conversation->appendPlainText(who + ": " + text.left(4096));
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
  // Leave the shared process channel on stdout for the accepted bridge.
  // Parse stderr only after QProcess has restored its own channel state.
  m_buffer += m_process->readAllStandardError();
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
}
void DirectPanel::reply(const QJsonObject &r) {
  if (r.value("kind") != "reply" || r.value("token").toString() != m_token)
    return;
  const auto phase = m_phase;
  m_token.clear();
  m_phase.clear();
  m_watchdog.stop();
  emit observed(phase, r);
  if (r.contains("providerState"))
    m_provider = r.value("providerState").toObject();
  if (r.value("protocol") == "provider.1") {
    m_provider = r;
    m_configured = r.value("ok").toBool();
    providerInformation();
    m_providerLabel->setText(m_remote
                                 ? "OpenRouter · NVIDIA free model"
                                 : "Offline · deterministic local provider");
    status(m_configured
               ? (m_remote && !m_provider.value("source")
                                   .toObject()
                                   .value("available")
                                   .toBool()
                      ? "Open Settings to choose a private credential source, "
                        "or use Offline."
                      : "Ready. Describe the tool you need.")
               : "Provider unavailable. Check Settings or select Offline.");
    refresh();
    return;
  }
  if (phase == "approve" && r.value("topologyCommitted").toBool()) {
    m_details->setPlainText(QString::fromUtf8(QJsonDocument(r).toJson()));
    const bool topologySaved = r.value("topologyCheckpointPersisted").toBool();
    QString message;
    if (!topologySaved)
      message = "The tool was added in memory, but its workspace could not be "
                "saved. Do not start a duplicate. Inspect Settings diagnostics "
                "before retrying.";
    else if (!r.value("initializationSucceeded").toBool())
      message = "The tool was added and its workspace saved, but the initial "
                "calculation failed. Edit its form to retry locally; do not "
                "start a duplicate.";
    else if (!r.value("checkpointPersisted").toBool())
      message = "The tool was added and its calculation validated, but the "
                "result could not be saved. Inspect storage diagnostics and "
                "use the form to recalculate.";
    else
      message = "Your tool is ready and saved on the canvas. Edits now run "
                "locally without the model.";
    status(message);
    conversation("Operator", message);
    m_workspace->activity(topologySaved ? "Tool workspace opened"
                                        : "Tool needs storage recovery",
                          r.value("roomId").toString());
    if (topologySaved)
      QTimer::singleShot(0, this, [this, id = r.value("roomId").toString()] {
        m_shell.bridge().selectRoom(id);
      });
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
    QString message;
    if (code == "AUTH_FAILED" || code == "NOT_CONFIGURED")
      message = "The provider could not authenticate. Check your private "
                "credential source in Settings, or use Offline.";
    else if (code == "RATE_LIMITED")
      message = "The provider is busy or has reached its quota. Try later, or "
                "choose Offline in Settings. No automatic retry was made.";
    else if (code == "MISSING_PARAMETER" || code == "INVALID_UNIT")
      message = "Some required values or units are missing. Include the tool's "
                "inputs in your request.";
    else if (code == "STATE_CONFLICT" || code == "APPROVAL_EXPIRED" ||
             code == "CATALOG_CHANGED")
      message = "The workspace or suggested tool changed. Ask again to review "
                "a fresh suggestion.";
    else if (code == "REFUSED")
      message = "The provider declined this request. Refine it or use Offline.";
    else if (code == "TIMEOUT" || code == "UNAVAILABLE" ||
             code == "PROVIDER_TIMEOUT")
      message = "The provider did not respond in time. Your workspace is "
                "unchanged. Try again or use Offline.";
    else
      message = "I could not validate a usable suggestion. Your workspace is "
                "unchanged. Refine the request or try Offline.";
    m_details->setPlainText(QString::fromUtf8(QJsonDocument(r).toJson()));
    status(message);
    conversation("Operator", message);
    m_workspace->activity("Request not completed");
    if (r.value("refreshDesktop").toBool())
      QTimer::singleShot(0, this, [this] {
        m_shell.bridge().selectRoom(m_shell.bridge().selectedRoom());
      });
    if (r.value("topologyCommitted").toBool()) {
      status("Tool added, but initialization or saving failed. Inspect the "
             "canvas and edit its form; do not start a duplicate.");
      if (r.value("topologyCheckpointPersisted").toBool())
        QTimer::singleShot(0, this, [this, id = r.value("roomId").toString()] {
          m_shell.bridge().selectRoom(id);
        });
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
    QStringList lines{WorkspaceView::toolName(component.value("id").toString()),
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
    status(
        "Review the suggested tool and inputs, then choose Start this tool.");
    conversation("Operator",
                 "Suggested " +
                     WorkspaceView::toolName(component.value("id").toString()) +
                     ". Review its values before starting.");
    refresh();
    return;
  }
  status(r.value("code") == "CANCELLED" ? "Request cancelled." : "Ready.");
  refresh();
}
