#include "DirectPanel.h"
#include "TraceRecorder.h"
#include <QApplication>
#include <QQuickStyle>
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  const auto a = app.arguments();
  if (a.size() < 7)
    return 2;
  bool live = false;
  QString trace;
  for (int i = 7; i < a.size(); ++i) {
    if (a[i] == "--live")
      live = true;
    else if (a[i] == "--trace" && i + 1 < a.size())
      trace = a[++i];
    else
      return 2;
  }
  DesktopShell shell({a[1], a[2], a[3], a[4], a[5], a[6]});
  DirectPanel panel(shell, live);
  std::unique_ptr<TraceRecorder> observer;
  if (!trace.isEmpty()) {
    observer = std::make_unique<TraceRecorder>(shell, panel, trace);
    if (!observer->valid())
      return 2;
  }
  QObject::connect(&shell, &DesktopShell::closed, &app,
                   [&app](bool ok) { app.exit(ok ? 0 : 1); });
  shell.show();
  return app.exec();
}
