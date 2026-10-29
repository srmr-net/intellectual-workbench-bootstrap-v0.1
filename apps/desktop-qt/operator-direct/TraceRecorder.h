#pragma once
#include "../desktop/DesktopShell.h"
#include <QFile>
#include <QJsonObject>
class DirectPanel;
// Explicit validation-only passive observer. It cannot issue host commands.
class TraceRecorder final : public QObject {
  Q_OBJECT
public:
  TraceRecorder(DesktopShell &, DirectPanel &, QString file,
                QObject *parent = nullptr);
  bool valid() const { return m_file.isOpen() && !m_failed; }
  void nativeEvent(const QString &, const QJsonObject &);

private:
  void state();
  void write(QJsonObject);
  DesktopShell &m_shell;
  DirectPanel &m_panel;
  QFile m_file;
  qint64 m_bytes = 0;
  bool m_failed = false;
};
