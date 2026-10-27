#include "OperatorDriver.h"
#include <QApplication>
#include <QQuickStyle>
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  const auto a = app.arguments();
  const bool validation = a.size() == 11 && a[7] == "--validation-actions" &&
                          a[9] == "--validation-report";
  if (a.size() != 7 && !validation)
    return 2;
  DesktopShell shell({a[1], a[2], a[3], a[4], a[5], a[6]});
  OperatorPanel panel(shell);
  QObject::connect(&shell, &DesktopShell::closed, &app,
                   [&app](bool ok) { app.exit(ok ? 0 : 1); });
  shell.show();
  std::unique_ptr<OperatorDriver> driver;
  if (validation) {
    driver = std::make_unique<OperatorDriver>(shell, panel, a[8], a[10]);
    driver->start();
  }
  const int code = app.exec();
  if (driver)
    driver->finish(code);
  return code;
}
