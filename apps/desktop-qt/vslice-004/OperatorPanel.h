#pragma once
#include "../desktop/DesktopShell.h"
#include <QElapsedTimer>
#include <QJsonObject>
#include <QSet>
#include <QTimer>
#include <QWidget>
class QComboBox;
class QCheckBox;
class QLabel;
class QPlainTextEdit;
class QPushButton;
class QProcess;
class QLineEdit;
class QTreeWidget;
class QProgressBar;
class OperatorPanel final : public QWidget {
  Q_OBJECT
public:
  explicit OperatorPanel(DesktopShell &);
  QJsonObject lastReply() const { return m_reply; }
  bool busy() const { return !m_current.isEmpty() || !m_pending.isEmpty(); }
  bool hasPreview() const { return !m_digest.isEmpty(); }
  QString error() const;
  bool guiThread() const;
  void inspect();
  QJsonObject providerState() const { return m_providerState; }
  QJsonObject disclosure() const { return m_review; }
  QJsonObject uxReply() const { return m_uxReply; }
  bool hasDisclosure() const { return !m_reviewId.isEmpty(); }
signals:
  void replyReceived();

private:
  void send(const QJsonObject &, bool provider = false, bool ux = false);
  void clearDisclosure();
  void publishStatus(const QString &);
  void showPreview(const QJsonObject &);
  void configureProvider();
  void updateProvider(const QJsonObject &);
  void read();
  void accept(const QJsonObject &);
  void clearPreview();
  void refresh();
  void fail(const QString &);
  DesktopShell &m_shell;
  QProcess *m_process;
  QPlainTextEdit *m_submitted = nullptr;
  QPlainTextEdit *m_intent, *m_preview;
  QCheckBox *m_consent;
  QPushButton *m_prepare, *m_approve, *m_decline, *m_cancel;
  QLabel *m_status;
  QByteArray m_buffer;
  QSet<QString> m_pending;
  QString m_current, m_digest, m_prefix, m_action;
  quint64 m_token = 0;
  QTimer m_deadline, m_watchdog;
  QJsonObject m_reply, m_providerState;
  QComboBox *m_providerChoice;
  QCheckBox *m_remoteConsent;
  QLabel *m_providerInfo;
  QLineEdit *m_sourcePath;
  QPushButton *m_sourceApply, *m_send;
  QPlainTextEdit *m_outbound;
  QTreeWidget *m_proposalRows;
  QProgressBar *m_progress;
  QTimer m_progressTimer, m_reviewDeadline;
  QElapsedTimer m_progressClock;
  QJsonObject m_review, m_uxReply;
  QString m_reviewId, m_reviewDigest, m_reviewRoom;
  bool m_selectingApproved = false;
};
