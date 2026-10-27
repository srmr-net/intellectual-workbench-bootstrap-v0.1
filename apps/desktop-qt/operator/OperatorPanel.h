#pragma once
#include "../desktop/DesktopShell.h"
#include <QJsonObject>
#include <QSet>
#include <QTimer>
#include <QWidget>
class QCheckBox;
class QLabel;
class QPlainTextEdit;
class QPushButton;
class QProcess;
class OperatorPanel final : public QWidget {
  Q_OBJECT
public:
  explicit OperatorPanel(DesktopShell &);
  QJsonObject lastReply() const { return m_reply; }
  bool busy() const { return !m_current.isEmpty(); }
  bool hasPreview() const { return !m_digest.isEmpty(); }
  QString error() const;
  bool guiThread() const;
  void inspect();
signals:
  void replyReceived();

private:
  void send(const QJsonObject &);
  void read();
  void accept(const QJsonObject &);
  void clearPreview();
  void refresh();
  void fail(const QString &);
  DesktopShell &m_shell;
  QProcess *m_process;
  QPlainTextEdit *m_intent, *m_preview;
  QCheckBox *m_consent;
  QPushButton *m_prepare, *m_approve, *m_decline, *m_cancel;
  QLabel *m_status;
  QByteArray m_buffer;
  QSet<QString> m_pending;
  QString m_current, m_digest, m_prefix, m_action;
  quint64 m_token = 0;
  QTimer m_deadline, m_watchdog;
  QJsonObject m_reply;
};
