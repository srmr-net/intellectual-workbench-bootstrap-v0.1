#pragma once
#include "DesktopBridge.h"
#include <QPointer>
#include <QQmlApplicationEngine>
#include <QWidget>
class QListWidget;
class QPushButton;
class QLabel;
class QQuickWindow;
class DesktopShell final : public QWidget {
  Q_OBJECT
public:
  DesktopShell(const DesktopConfig &, int capacity = 32,
               QWidget *parent = nullptr);
  DesktopBridge &bridge() { return m_bridge; }
  QQuickWindow *canvasWindow() const { return m_canvas; }
  void decorateFields();

signals:
  void closed(bool ok);

protected:
  void closeEvent(QCloseEvent *) override;
  bool eventFilter(QObject *, QEvent *) override;

private:
  void refresh();
  DesktopBridge m_bridge;
  QQmlApplicationEngine m_engine;
  QQuickWindow *m_canvas = nullptr;
  QListWidget *m_installed, *m_rooms;
  QPushButton *m_open, *m_pin, *m_quit;
  QLabel *m_status, *m_identity, *m_error;
  QWidget *m_container = nullptr;
  QVariantList m_previousInstalled, m_previousRooms;
  bool m_shutdown = false;
};
