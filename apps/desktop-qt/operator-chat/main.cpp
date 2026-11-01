#include "ChatWindow.h"
#include <QApplication>
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  const auto a = app.arguments();
  if (a.size() != 3 && !(a.size() == 4 && a[3] == "--fixture"))
    return 2;
  ChatWindow window(a[1], a[2], a.size() == 4);
  window.show();
  return app.exec();
}
