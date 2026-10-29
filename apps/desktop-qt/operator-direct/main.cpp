#include "DirectPanel.h"
#include <QApplication>
#include <QQuickStyle>
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  const auto a = app.arguments();
  const bool live = a.size() == 8 && a[7] == "--live";
  if (a.size() != 7 && !live)
    return 2;
  DesktopShell shell({a[1], a[2], a[3], a[4], a[5], a[6]});
  DirectPanel panel(shell, live);
  QObject::connect(&shell, &DesktopShell::closed, &app,
                   [&app](bool ok) { app.exit(ok ? 0 : 1); });
  shell.show();
  return app.exec();
}
