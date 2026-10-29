#pragma once
#include "../desktop/DesktopShell.h"
#include <QJsonObject>
#include <QTimer>
#include <QWidget>
class QLabel;
class QPlainTextEdit;
class QPushButton;
class QProcess;
class QGroupBox;
class DirectPanel final : public QWidget {
  Q_OBJECT
public:
  DirectPanel(DesktopShell &, bool live);
  bool busy() const { return !m_token.isEmpty(); }
  QJsonObject providerState() const { return m_provider; }

private:
  void send(QString protocol, QJsonObject command);
  void read();
  void reply(const QJsonObject &);
  void configure();
  void settings();
  void status(QString);
  void refresh();
  void clearProposal();
  DesktopShell &m_shell;
  QProcess *m_process;
  QPlainTextEdit *m_request, *m_details;
  QLabel *m_status, *m_providerLabel, *m_summary;
  QPushButton *m_ask, *m_start, *m_reject, *m_cancel, *m_settings;
  QGroupBox *m_card;
  QByteArray m_buffer;
  QString m_prefix, m_token, m_phase, m_digest, m_file;
  quint64 m_sequence = 0;
  bool m_remote = false, m_configured = false, m_sourceThenConfigure = false;
  QJsonObject m_provider, m_preview;
  QTimer m_expiry, m_watchdog;
};
