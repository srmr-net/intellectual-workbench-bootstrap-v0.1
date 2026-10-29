#pragma once
#include <QHash>
#include <QJsonObject>
#include <QProcess>
#include <QWidget>
class QLabel;
class QListWidget;
class QPlainTextEdit;
class QPushButton;
class QTextBrowser;
class ChatWindow final : public QWidget {
  Q_OBJECT
public:
  ChatWindow(QString worker, QString toolsLauncher, bool fixture = false);
  QJsonObject state() const { return m_state; }
signals:
  void stateChanged();

protected:
  bool eventFilter(QObject *, QEvent *) override;
  void closeEvent(QCloseEvent *) override;

private:
  void command(QJsonObject, bool request = false);
  void read();
  void frame(const QJsonObject &);
  void render();
  void settings();
  void send();
  void status(QString);
  QProcess m_process;
  QByteArray m_buffer;
  QJsonObject m_state;
  QHash<QString, QString> m_drafts;
  QString m_control, m_requestToken, m_path, m_tools, m_sentText;
  quint64 m_sequence = 0;
  bool m_initialized = false, m_ready = false, m_closing = false,
       m_closed = false, m_fixture = false;
  QLabel *m_status, *m_provider, *m_welcome, *m_limit;
  QListWidget *m_chats;
  QTextBrowser *m_transcript;
  QPlainTextEdit *m_composer;
  QPushButton *m_send, *m_stop, *m_retry, *m_new;
  QByteArray m_lastMessages;
  QString m_lastSession;
};
