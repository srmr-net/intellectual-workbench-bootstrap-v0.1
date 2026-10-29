#pragma once
#include "../desktop/DesktopShell.h"
#include <QObject>
class QLabel;
class QListWidget;
class QPlainTextEdit;
class QStackedWidget;
class WorkspaceView final : public QObject {
  Q_OBJECT
public:
  WorkspaceView(DesktopShell &, QWidget *componentsPage, QObject *parent);
  void activity(const QString &, const QString &roomId = {});
  static QString toolName(QString);

private:
  void refresh();
  void componentInformation();
  DesktopShell &m_shell;
  QListWidget *m_workspaces, *m_installed, *m_pinned, *m_recent;
  QLabel *m_canvasTitle, *m_empty, *m_workspaceStatus;
  QPlainTextEdit *m_componentInfo;
  QStackedWidget *m_canvasStack;
  QString m_registry, m_selected;
  QVariantList m_lastRooms;
};
