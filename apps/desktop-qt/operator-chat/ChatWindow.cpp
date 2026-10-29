#include "ChatWindow.h"
#include <QAccessible>
#include <QApplication>
#include <QCheckBox>
#include <QCloseEvent>
#include <QDialog>
#include <QDialogButtonBox>
#include <QDir>
#include <QFileInfo>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QScrollBar>
#include <QSignalBlocker>
#include <QTabWidget>
#include <QTextBrowser>
#include <QTextCursor>
#include <QTimer>
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
QPushButton *button(QString text, const char *name, QWidget *parent) {
  auto *b = new QPushButton(text, parent);
  b->setObjectName(name);
  b->setAccessibleName(text);
  return b;
}
QString errorMessage(QString code) {
  if (code == "CONSENT_REQUIRED")
    return "Enable OpenRouter in Settings to chat, or choose the offline "
           "fixture.";
  if (code == "NOT_CONFIGURED" || code == "AUTH_FAILED")
    return "The provider credential is unavailable or was rejected. Check its "
           "private file reference in Settings.";
  if (code == "RATE_LIMITED")
    return "OpenRouter is rate limited. Your message is kept here; you can "
           "retry later.";
  if (code == "TIMEOUT" || code == "UNAVAILABLE")
    return "The provider did not respond. Your conversation is kept; no "
           "automatic retry was made.";
  if (code == "REFUSED")
    return "The provider declined this message. You can rephrase it.";
  if (code == "TRUNCATED")
    return "The provider reached the response limit. No partial answer was "
           "added; try a narrower question.";
  if (code == "BUDGET_EXCEEDED")
    return "A message, conversation or request limit was reached. Shorten your "
           "message or start a new chat.";
  if (code == "CANCELLED")
    return "Stopped. You can continue the conversation.";
  return "The provider returned an unusable answer. Nothing was executed; you "
         "can retry or rephrase.";
}
} // namespace
ChatWindow::ChatWindow(QString worker, QString toolsLauncher, bool fixture)
    : m_tools(std::move(toolsLauncher)), m_fixture(fixture) {
  setWindowTitle("Intellectual Workbench — Operator");
  setObjectName("operatorChatWindow");
  resize(1160, 800);
  m_path = qEnvironmentVariable("WB_LLM_SECRET_FILE");
  auto *columns = new QHBoxLayout(this);
  columns->setContentsMargins(18, 18, 18, 18);
  columns->setSpacing(24);
  auto *sidebar = new QWidget(this);
  sidebar->setFixedWidth(210);
  auto *nav = new QVBoxLayout(sidebar);
  nav->setContentsMargins(0, 0, 0, 0);
  auto *brand = label("Workbench", sidebar);
  auto font = brand->font();
  font.setPointSize(font.pointSize() + 4);
  font.setBold(true);
  brand->setFont(font);
  nav->addWidget(brand);
  m_new = button("New chat", "chatNew", sidebar);
  m_new->setToolTip("Starting or switching chats stops the current response.");
  nav->addWidget(m_new);
  nav->addWidget(label("Conversations", sidebar));
  m_chats = new QListWidget(sidebar);
  m_chats->setObjectName("chatSessions");
  m_chats->setAccessibleName("Conversations in this session");
  m_chats->setToolTip("Switching chats stops the current response.");
  m_chats->setAccessibleDescription(
      "Switching chats stops the current response.");
  nav->addWidget(m_chats, 1);
  auto *settingsButton = button("Settings", "chatSettings", sidebar);
  nav->addWidget(settingsButton);
  columns->addWidget(sidebar);
  auto *main = new QWidget(this);
  auto *layout = new QVBoxLayout(main);
  layout->setContentsMargins(0, 0, 0, 0);
  layout->setSpacing(12);
  auto *title = label("Operator", main);
  title->setFont(font);
  layout->addWidget(title);
  m_provider = label("Connecting to the local chat service…", main);
  m_provider->setObjectName("chatProvider");
  layout->addWidget(m_provider);
  m_welcome = label("What would you like to work on?\nTalk through an idea, "
                    "ask a question, or tell Operator what you want to do.",
                    main);
  m_welcome->setObjectName("chatWelcome");
  layout->addWidget(m_welcome);
  m_transcript = new QTextBrowser(main);
  m_transcript->setObjectName("chatTranscript");
  m_transcript->setAccessibleName("Operator conversation");
  m_transcript->setOpenLinks(false);
  m_transcript->setOpenExternalLinks(false);
  auto readingFont = m_transcript->font();
  readingFont.setPointSizeF(readingFont.pointSizeF() + 2);
  m_transcript->setFont(readingFont);
  m_transcript->document()->setDocumentMargin(14);
  m_transcript->setReadOnly(true);
  m_transcript->setFrameShape(QFrame::NoFrame);
  layout->addWidget(m_transcript, 1);
  m_status = label("Opening conversation…", main);
  m_status->setObjectName("chatStatus");
  m_status->setAccessibleName("Conversation status");
  layout->addWidget(m_status);
  m_composer = new QPlainTextEdit(main);
  m_composer->setObjectName("chatComposer");
  m_composer->setAccessibleName("Message Operator");
  m_composer->setPlaceholderText("Message Operator…");
  m_composer->setMinimumHeight(85);
  m_composer->setMaximumHeight(140);
  m_composer->setTabChangesFocus(true);
  m_composer->installEventFilter(this);
  layout->addWidget(m_composer);
  m_limit = label(
      "This message is too long. Shorten it to send (maximum 4 KB of text).",
      main);
  m_limit->setObjectName("chatMessageLimit");
  m_limit->hide();
  layout->addWidget(m_limit);
  auto *actions = new QHBoxLayout;
  actions->addWidget(label("Enter to send · Shift+Enter for a new line", main));
  actions->addStretch();
  m_retry = button("Edit last message", "chatRetry", main);
  m_retry->hide();
  actions->addWidget(m_retry);
  m_stop = button("Stop", "chatStop", main);
  m_stop->hide();
  actions->addWidget(m_stop);
  m_send = button("Send", "chatSend", main);
  actions->addWidget(m_send);
  layout->addLayout(actions);
  layout->addWidget(label("Recent context: up to 12 completed turns · chat "
                          "history stays in this application session",
                          main));
  columns->addWidget(main, 1);
  connect(settingsButton, &QPushButton::clicked, this, &ChatWindow::settings);
  connect(m_new, &QPushButton::clicked, this,
          [this] { command({{"type", "new"}}); });
  connect(m_chats, &QListWidget::itemClicked, this,
          [this](QListWidgetItem *item) {
            if (item->data(Qt::UserRole).toString() !=
                m_state.value("selected").toString())
              command({{"type", "select"},
                       {"sessionId", item->data(Qt::UserRole).toString()}});
          });
  connect(m_chats, &QListWidget::itemActivated, this,
          [this](QListWidgetItem *item) {
            if (item->data(Qt::UserRole).toString() !=
                m_state.value("selected").toString())
              command({{"type", "select"},
                       {"sessionId", item->data(Qt::UserRole).toString()}});
          });
  connect(m_send, &QPushButton::clicked, this, &ChatWindow::send);
  connect(m_stop, &QPushButton::clicked, this, [this] {
    command({{"type", "cancel"}});
    status("Stopped. You can continue the conversation.");
  });
  connect(m_retry, &QPushButton::clicked, this, [this] {
    const auto messages = m_state.value("messages").toArray();
    if (!messages.isEmpty()) {
      const auto last = messages.last().toObject();
      if (last.value("role") == "user" && last.value("status") == "failed") {
        m_composer->setPlainText(last.value("text").toString());
        m_composer->setFocus();
      }
    }
  });
  connect(m_composer, &QPlainTextEdit::textChanged, this, [this] { render(); });
  connect(&m_process, &QProcess::readyReadStandardOutput, this,
          &ChatWindow::read);
  connect(&m_process, &QProcess::errorOccurred, this, [this] {
    m_ready = false;
    status("The local chat service is unavailable. Restart Workbench; no "
           "automatic request was made.");
    render();
  });
  connect(&m_process, qOverload<int, QProcess::ExitStatus>(&QProcess::finished),
          this, [this](int, QProcess::ExitStatus) {
            m_ready = false;
            if (m_closing) {
              m_closed = true;
              close();
            } else {
              status("The local chat service stopped. Your visible "
                     "conversation is retained.");
              render();
            }
          });
  m_process.setProcessChannelMode(QProcess::SeparateChannels);
  // Never copy child stderr (which could contain third-party text) into
  // UI/logs.
  connect(&m_process, &QProcess::readyReadStandardError, this,
          [this] { m_process.readAllStandardError(); });
  m_process.start(worker, fixture ? QStringList{"--fixture"} : QStringList{});
  render();
}
void ChatWindow::status(QString text) {
  m_status->setText(text);
  m_status->setAccessibleName(text);
  m_status->setAccessibleDescription(text);
  QAccessibleEvent event(m_status, QAccessible::Alert);
  QAccessible::updateAccessibility(&event);
}
bool ChatWindow::eventFilter(QObject *object, QEvent *event) {
  if (object == m_composer && event->type() == QEvent::KeyPress) {
    auto *key = static_cast<QKeyEvent *>(event);
    if ((key->key() == Qt::Key_Return || key->key() == Qt::Key_Enter) &&
        !(key->modifiers() & Qt::ShiftModifier)) {
      if (!key->isAutoRepeat())
        m_send->click();
      return true;
    }
  }
  return QWidget::eventFilter(object, event);
}
void ChatWindow::send() {
  if (!m_send->isEnabled())
    return;
  m_sentText = m_composer->toPlainText();
  m_retry->hide();
  command({{"type", "send"},
           {"sessionId", m_state.value("selected")},
           {"text", m_sentText}},
          true);
  status("Operator is thinking… You can keep writing. Stop or switch chats to "
         "end this response.");
}
void ChatWindow::command(QJsonObject command, bool request) {
  if (m_process.state() != QProcess::Running)
    return;
  const auto token = "chat-" + QString::number(++m_sequence);
  if (request)
    m_requestToken = token;
  else {
    m_control = token;
    m_requestToken.clear();
  }
  const auto bytes = QJsonDocument(QJsonObject{{"protocol", "chat.1"},
                                               {"token", token},
                                               {"command", command}})
                         .toJson(QJsonDocument::Compact);
  if (bytes.size() > 32768) {
    m_requestToken.clear();
    m_control.clear();
    status("Please shorten this message.");
    render();
    return;
  }
  QByteArray prefix(4, '\0');
  qToBigEndian<quint32>(bytes.size(), prefix.data());
  if (m_process.write(prefix + bytes) != prefix.size() + bytes.size()) {
    status("Could not contact the local chat service.");
    m_process.kill();
  }
  render();
}
void ChatWindow::read() {
  m_buffer += m_process.readAllStandardOutput();
  if (m_buffer.size() > 4 * 1024 * 1024 + 4) {
    m_process.kill();
    return;
  }
  while (m_buffer.size() >= 4) {
    const auto n = qFromBigEndian<quint32>(m_buffer.constData());
    if (!n || n > 4 * 1024 * 1024) {
      m_process.kill();
      return;
    }
    if (m_buffer.size() < int(n) + 4)
      return;
    QJsonParseError error;
    const auto document = QJsonDocument::fromJson(m_buffer.mid(4, n), &error);
    m_buffer.remove(0, n + 4);
    if (error.error != QJsonParseError::NoError || !document.isObject()) {
      m_process.kill();
      return;
    }
    frame(document.object());
  }
}
void ChatWindow::frame(const QJsonObject &reply) {
  if (reply.value("protocol") != "chat.1") {
    m_process.kill();
    return;
  }
  const auto kind = reply.value("kind").toString(),
             token = reply.value("token").toString();
  if (kind == "fatal") {
    status("The local chat service rejected an invalid message.");
    m_process.kill();
    return;
  }
  if (kind == "ready" && !m_initialized) {
    m_initialized = true;
    m_ready = true;
    m_state = reply.value("state").toObject();
    command({{"type", "configure"},
             {"enabled", false},
             {"credentialPath", m_path.isEmpty() ? QJsonValue(QJsonValue::Null)
                                                 : QJsonValue(m_path)}});
    return;
  }
  if (token != m_control && token != m_requestToken)
    return; // cancelled/superseded generation
  m_state = reply.value("state").toObject();
  if (kind == "started") {
    if (m_composer->toPlainText() == m_sentText)
      m_composer->clear();
  } else {
    if (token == m_control)
      m_control.clear();
    if (token == m_requestToken)
      m_requestToken.clear();
    if (kind == "error") {
      status(errorMessage(reply.value("code").toString()));

    } else if (m_state.value("selected").toString().isEmpty() && !m_closing) {
      command({{"type", "new"}});
      return;
    } else if (kind == "completed") {
      const auto usage = m_state.value("usage").toObject();
      status(
          "Reply received · " +
          QString::number(usage.value("elapsedMs").toDouble() / 1000., 'f', 1) +
          "s");
    } else if (!m_state.value("enabled").toBool())
      status("Enable OpenRouter once in Settings to start chatting. Your draft "
             "stays here.");
    else
      status("Ready. What would you like to work on?");
  }
  render();
  emit stateChanged();
}
void ChatWindow::render() {
  const auto selected = m_state.value("selected").toString();
  if (selected != m_lastSession && !selected.isEmpty()) {
    if (!m_lastSession.isEmpty()) {
      m_drafts[m_lastSession] = m_composer->toPlainText();
      QSignalBlocker block(m_composer);
      m_composer->setPlainText(m_drafts.value(selected));
    }
    m_composer->setFocus();
  }
  const bool busy = m_state.value("busy").toBool() || !m_requestToken.isEmpty();
  const bool controls = m_ready && m_control.isEmpty() && !m_closing;
  // Keep focus; do not assign keystrokes to the old chat during a switch ACK.
  m_composer->setReadOnly(!m_ready || !m_control.isEmpty() || m_closing);
  const bool tooLong = m_composer->toPlainText().toUtf8().size() > 4096;
  if (tooLong != m_limit->isVisible()) {
    m_limit->setVisible(tooLong);
    if (tooLong) {
      QAccessibleEvent alert(m_limit, QAccessible::Alert);
      QAccessible::updateAccessibility(&alert);
    }
  }
  m_send->setEnabled(controls && !busy && m_state.value("enabled").toBool() &&
                     m_state.value("credentialAvailable").toBool() &&
                     !m_composer->toPlainText().trimmed().isEmpty() &&
                     m_composer->toPlainText().toUtf8().size() <= 4096);
  m_stop->setVisible(busy);
  m_stop->setEnabled(controls);
  m_new->setEnabled(controls);
  m_chats->setEnabled(controls);
  m_provider->setText(m_state.value("available").toBool()
                          ? (m_state.value("enabled").toBool()
                                 ? "OpenRouter · NVIDIA Nemotron 3 Ultra"
                                 : "OpenRouter is off · enable it in Settings")
                          : "Offline fixture / unavailable remote transport");
  if (m_state.value("enabled").toBool() &&
      !m_state.value("credentialAvailable").toBool())
    m_provider->setText("Credential unavailable · check Settings");
  if (m_fixture)
    m_provider->setText("Offline chat fixture · no remote requests");
  m_provider->setToolTip(m_state.value("model").toString());
  const auto sessions = m_state.value("sessions").toArray();
  {
    QSignalBlocker block(m_chats);
    QStringList existing;
    for (int i = 0; i < m_chats->count(); ++i)
      existing << m_chats->item(i)->data(Qt::UserRole).toString();
    QStringList ids;
    for (const auto &s : sessions)
      ids << s.toObject().value("id").toString();
    if (ids != existing) {
      m_chats->clear();
      for (const auto &s : sessions) {
        auto *item = new QListWidgetItem(s.toObject().value("title").toString(),
                                         m_chats);
        item->setData(Qt::UserRole, s.toObject().value("id"));
      }
    }
    for (int i = 0; i < sessions.size(); ++i) {
      m_chats->item(i)->setText(
          sessions[i].toObject().value("title").toString());
      if (m_chats->item(i)->data(Qt::UserRole).toString() == selected)
        m_chats->setCurrentRow(i);
    }
  }
  const auto messages = m_state.value("messages").toArray();
  const auto encoded = QJsonDocument(messages).toJson(QJsonDocument::Compact);
  m_retry->setVisible(!busy && !messages.isEmpty() &&
                      messages.last().toObject().value("role") == "user" &&
                      messages.last().toObject().value("status") == "failed");
  m_welcome->setVisible(messages.isEmpty());
  if (encoded != m_lastMessages || selected != m_lastSession) {
    auto *scroll = m_transcript->verticalScrollBar();
    const int position = scroll->value();
    const bool bottom =
        position >= scroll->maximum() - 4 || selected != m_lastSession;
    m_transcript->clear();
    QTextCursor cursor(m_transcript->document());
    for (const auto &entry : messages) {
      const auto m = entry.toObject();
      QTextCharFormat heading;
      heading.setFontWeight(QFont::Bold);
      cursor.insertText(m.value("role") == "user" ? "You\n" : "Operator\n",
                        heading);
      QTextCharFormat body;
      cursor.insertText(m.value("text").toString() + "\n", body);
      if (m.value("status") != "complete") {
        QTextCharFormat meta;
        meta.setFontItalic(true);
        cursor.insertText("[" + m.value("status").toString() + "]\n", meta);
      }
      cursor.insertText("\n", body);
    }
    scroll->setValue(bottom ? scroll->maximum() : position);
    m_lastMessages = encoded;
    m_lastSession = selected;
  }
}
void ChatWindow::settings() {
  QDialog dialog(this);
  dialog.setObjectName("chatSettingsDialog");
  dialog.setWindowTitle("Workbench Settings");
  dialog.resize(610, 520);
  auto *layout = new QVBoxLayout(&dialog);
  auto *tabs = new QTabWidget(&dialog);
  layout->addWidget(tabs);
  auto *provider = new QWidget(tabs);
  auto *p = new QVBoxLayout(provider);
  auto *enabled =
      new QCheckBox("Use OpenRouter for this application session", provider);
  enabled->setObjectName("chatProviderEnabled");
  enabled->setChecked(m_state.value("enabled").toBool());
  p->addWidget(enabled);
  p->addWidget(
      label("Send transmits your message and up to the last 12 completed "
            "conversation turns to OpenRouter and NVIDIA. No tool registry, "
            "room state or local files are included. NVIDIA's free endpoint "
            "logs requests for security and product improvement. Chat replies "
            "cannot execute actions.",
            provider));
  p->addWidget(
      label("Model: nvidia/nemotron-3-ultra-550b-a55b:free\nEndpoint: "
            "https://openrouter.ai/api/v1/chat/completions\nNVIDIA only · no "
            "fallback · zero-price routing · no tools/retries\nEach Send: at "
            "most 1024 output tokens, 30 seconds",
            provider));
  auto *sourceLabel = label("Private credential file reference (optional; "
                            "otherwise host environment)",
                            provider);
  p->addWidget(sourceLabel);
  auto *source = new QLineEdit(m_path, provider);
  source->setObjectName("chatCredentialSource");
  source->setAccessibleName("Private credential file reference");
  sourceLabel->setBuddy(source);
  p->addWidget(source);
  p->addWidget(label("Enter a path, never the API key itself. Credentials are "
                     "loaded only by the native host; this setting and chat "
                     "history are not written to room snapshots.",
                     provider));
  p->addStretch();
  tabs->addTab(provider, "Provider");
  auto *tools = new QWidget(tabs);
  auto *t = new QVBoxLayout(tools);
  t->addWidget(
      label("The existing component workspace remains available separately. "
            "Chat has no component execution authority. Use its validated "
            "suggestion and Start controls to open a tool.",
            tools));
  auto *open = button("Open tools workspace", "chatOpenTools", tools);
  open->setEnabled(QFileInfo::exists(m_tools));
  t->addWidget(open);
  t->addStretch();
  tabs->addTab(tools, "Tools");
  connect(open, &QPushButton::clicked, &dialog, [this, &dialog] {
    if (QProcess::startDetached(m_tools, {}))
      dialog.reject();
    else
      status("Could not open the tools workspace. Build the existing desktop "
             "first.");
  });
  auto *buttons = new QDialogButtonBox(
      QDialogButtonBox::Save | QDialogButtonBox::Cancel, &dialog);
  buttons->setObjectName("chatSettingsButtons");
  layout->addWidget(buttons);
  connect(buttons, &QDialogButtonBox::accepted, &dialog, &QDialog::accept);
  connect(buttons, &QDialogButtonBox::rejected, &dialog, &QDialog::reject);
  if (dialog.exec() == QDialog::Accepted) {
    m_path = source->text().trimmed();
    command({{"type", "configure"},
             {"enabled", enabled->isChecked()},
             {"credentialPath", m_path.isEmpty() ? QJsonValue(QJsonValue::Null)
                                                 : QJsonValue(m_path)}});
  }
  m_composer->setFocus();
}
void ChatWindow::closeEvent(QCloseEvent *event) {
  if (m_closed || m_process.state() == QProcess::NotRunning) {
    event->accept();
    return;
  }
  event->ignore();
  if (m_closing)
    return;
  m_closing = true;
  command({{"type", "quit"}});
  QTimer::singleShot(3000, this, [this] {
    if (m_process.state() != QProcess::NotRunning)
      m_process.kill();
  });
}
