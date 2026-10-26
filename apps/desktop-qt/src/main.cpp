#include "UiBridge.h"
#include <QGuiApplication>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickStyle>
int main(int argc, char **argv) {
  QGuiApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  const auto a = app.arguments();
  if (a.size() != 8 && a.size() != 9)
    return 2;
  UiBridge bridge;
  QQmlApplicationEngine engine;
  engine.rootContext()->setContextProperty("bridge", &bridge);
  engine.load(QUrl("qrc:/qml/Main.qml"));
  if (engine.rootObjects().isEmpty())
    return 1;
  LaunchConfig c{a[1], a[2], a[3], a[4],
                 a[5], a[6], a[7], a.size() == 9 && a[8] == "restore"};
  bridge.start(c);
  QObject::connect(&app, &QCoreApplication::aboutToQuit, &bridge,
                   &UiBridge::stop);
  return app.exec();
}
