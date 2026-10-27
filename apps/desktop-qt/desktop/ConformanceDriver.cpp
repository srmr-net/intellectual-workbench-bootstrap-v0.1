#include "ConformanceDriver.h"
#include <QAccessible>
#include <QAccessibleInterface>
#include <QApplication>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QJsonDocument>
#include <QListWidget>
#include <QPushButton>
#include <QQuickItem>
#include <QQuickWindow>
#include <QtTest>
namespace {
QQuickItem *find(QQuickItem *root, const QString &name) {
  if (root->objectName() == name)
    return root;
  for (auto *c : root->childItems())
    if (auto *v = find(c, name))
      return v;
  return nullptr;
}
} // namespace
ConformanceDriver::ConformanceDriver(DesktopShell &shell, QString actions,
                                     QString report)
    : m_shell(shell), m_actions(std::move(actions)),
      m_report(std::move(report)) {}
void ConformanceDriver::start() {
  m_deadline.start();
  QTimer::singleShot(100, &m_shell, [this] { run(); });
}
bool ConformanceDriver::wait(const std::function<bool()> &predicate, int ms) {
  QElapsedTimer timer;
  timer.start();
  while (!predicate()) {
    if (timer.elapsed() > ms || m_deadline.elapsed() > 120000)
      return false;
    QTest::qWait(20);
  }
  return true;
}
void ConformanceDriver::sample(const QString &action) {
  QJsonArray controls;
  auto &bridge = m_shell.bridge();
  for (const auto &entry : bridge.controls()) {
    const auto control = entry.toMap();
    const QString prefix =
        control.value("type") == "readout" ? "output_" : "input_";
    auto *item = find(m_shell.canvasWindow()->contentItem(),
                      prefix + control.value("id").toString());
    if (!item)
      continue;
    auto *accessible = QAccessible::queryAccessibleInterface(item);
    controls.append(QJsonObject{
        {"id", control.value("id").toString()},
        {"text", item->property("text").toString()},
        {"role", accessible ? int(accessible->role()) : -1},
        {"name", accessible ? accessible->text(QAccessible::Name) : QString()},
        {"description",
         accessible ? accessible->text(QAccessible::Description) : QString()},
        {"focusable", accessible ? bool(accessible->state().focusable) : false},
        {"focus", item->hasActiveFocus()}});
  }
  QJsonArray columns;
  for (const QString &name :
       {QString("leftPinnedTools"), QString("activityContext"),
        QString("semanticCanvas")}) {
    const auto *widget = m_shell.findChild<QWidget *>(name);
    if (widget)
      columns.append(QJsonObject{{"name", name},
                                 {"x", widget->geometry().x()},
                                 {"width", widget->width()},
                                 {"visible", widget->isVisible()}});
  }
  m_samples.append(
      QJsonObject{{"action", action},
                  {"state", bridge.lastState()},
                  {"error", bridge.error()},
                  {"controls", controls},
                  {"columns", columns},
                  {"guiThread", bridge.callbacksOnGuiThread()},
                  {"windowActive", m_shell.canvasWindow()->isActive()},
                  {"windowVisible", m_shell.canvasWindow()->isVisible()},
                  {"ready", bridge.ready()},
                  {"pending", bridge.pendingCount()}});
}
void ConformanceDriver::run() {
  auto fail = [this](const QString &error) {
    m_failure = error;
    sample("FAILED");
    QCoreApplication::exit(3);
  };
  QFile file(m_actions);
  if (!file.open(QIODevice::ReadOnly) || file.size() > 65536) {
    fail("invalid action file");
    return;
  }
  QJsonParseError parse;
  const auto doc = QJsonDocument::fromJson(file.readAll(), &parse);
  if (parse.error != QJsonParseError::NoError || !doc.isArray() ||
      doc.array().size() > 128) {
    fail("invalid action plan");
    return;
  }
  auto &bridge = m_shell.bridge();
  if (!wait([&] { return !bridge.lastState().isEmpty(); })) {
    fail("desktop ready timeout");
    return;
  }
  for (const auto &entry : doc.array()) {
    const auto action = entry.toObject();
    const QString type = action.value("type").toString();
    if (type == "open") {
      auto *list = m_shell.findChild<QListWidget *>("installedComponents");
      int selected = -1;
      for (int i = 0; i < list->count(); ++i)
        if (action.value("id").toString().isEmpty() ||
            list->item(i)->data(Qt::UserRole).toMap().value("id") ==
                action.value("id").toString()) {
          selected = i;
          break;
        }
      if (selected < 0) {
        fail("installed component unavailable");
        return;
      }
      list->setCurrentRow(selected);
      QTest::mouseClick(m_shell.findChild<QPushButton *>("openComponent"),
                        Qt::LeftButton);
      if (!wait([&] { return !bridge.switching() && !bridge.busy(); })) {
        fail("open timeout");
        return;
      }
    } else if (type == "input") {
      auto *item = find(m_shell.canvasWindow()->contentItem(),
                        "input_" + action.value("id").toString());
      if (!item) {
        fail("input unavailable");
        return;
      }
      m_shell.findChild<QWidget *>("semanticCanvas")
          ->setFocus(Qt::OtherFocusReason);
      m_shell.canvasWindow()->requestActivate();
      QTest::qWait(30);
      item->forceActiveFocus();
      if (!item->hasActiveFocus()) {
        fail("semantic editor focus unavailable");
        return;
      }
      QTest::keyClick(m_shell.canvasWindow(), Qt::Key_A, Qt::ControlModifier);
      for (char c : action.value("text").toString().toLatin1())
        QTest::keyClick(m_shell.canvasWindow(), c);
      if (action.value("commit").toBool(true))
        QTest::keyClick(m_shell.canvasWindow(), Qt::Key_Tab);
      if (action.value("settle").toBool(true) &&
          !wait([&] { return !bridge.busy(); })) {
        fail("edit timeout");
        return;
      }
      if (action.contains("tabTo") &&
          (!m_shell.canvasWindow()->activeFocusItem() ||
           m_shell.canvasWindow()->activeFocusItem()->objectName() !=
               "input_" + action.value("tabTo").toString())) {
        fail("tab order");
        return;
      }
    } else if (type == "focus") {
      auto *item = find(m_shell.canvasWindow()->contentItem(),
                        "input_" + action.value("id").toString());
      if (!item) {
        fail("focus input missing");
        return;
      }
      m_shell.findChild<QWidget *>("semanticCanvas")
          ->setFocus(Qt::OtherFocusReason);
      m_shell.canvasWindow()->requestActivate();
      QTest::qWait(30);
      item->forceActiveFocus();
      item->setProperty("cursorPosition", action.value("cursor").toInt());
    } else if (type == "backgroundEdit") {
      bridge.edit(action.value("id").toString(),
                  action.value("text").toString(), bridge.viewEpoch());
      if (!wait([&] { return !bridge.busy(); })) {
        fail("background edit timeout");
        return;
      }
    } else if (type == "apply") {
      auto *item = find(m_shell.canvasWindow()->contentItem(), "applyDraft");
      if (!item) {
        fail("apply unavailable");
        return;
      }
      const auto point =
          item->mapToScene(QPointF(item->width() / 2, item->height() / 2))
              .toPoint();
      QTest::mouseClick(m_shell.canvasWindow(), Qt::LeftButton, Qt::NoModifier,
                        point);
      if (!wait([&] { return !bridge.busy(); })) {
        fail("apply timeout");
        return;
      }
    } else if (type == "pin") {
      QTest::mouseClick(m_shell.findChild<QPushButton *>("pinCanvas"),
                        Qt::LeftButton);
      if (!wait([&] { return !bridge.busy(); })) {
        fail("pin timeout");
        return;
      }
    } else if (type == "select") {
      auto *list = m_shell.findChild<QListWidget *>("pinnedRooms");
      const int index = action.value("index").toInt(-1);
      if (index < 0 || index >= list->count()) {
        fail("room unavailable");
        return;
      }
      QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                        list->visualItemRect(list->item(index)).center());
      if (!wait([&] { return !bridge.switching() && !bridge.busy(); })) {
        fail("select timeout");
        return;
      }
    } else if (type == "expect") {
      const auto values = action.value("values").toObject();
      for (auto it = values.constBegin(); it != values.constEnd(); ++it)
        if (!wait(
                [&] {
                  return bridge.committedValue(it.key()) ==
                         it.value().toString();
                },
                15000)) {
          fail("committed value mismatch: " + it.key());
          return;
        }
      for (auto it = values.constBegin(); it != values.constEnd(); ++it) {
        QString prefix;
        for (const auto &v : bridge.controls())
          if (v.toMap().value("id") == it.key())
            prefix =
                v.toMap().value("type") == "readout" ? "output_" : "input_";
        auto *item = prefix.isEmpty()
                         ? nullptr
                         : find(m_shell.canvasWindow()->contentItem(),
                                prefix + it.key());
        if (!item || !wait(
                         [&] {
                           return item->property("text").toString() ==
                                  it.value().toString();
                         },
                         15000)) {
          fail("rendered widget value mismatch: " + it.key());
          return;
        }
      }
      if (action.contains("focus")) {
        auto *focus = m_shell.canvasWindow()->activeFocusItem();
        if (!focus ||
            focus->objectName() !=
                "input_" + action.value("focus").toString() ||
            (action.contains("cursor") &&
             focus->property("cursorPosition").toInt() !=
                 action.value("cursor").toInt())) {
          fail("focus/cursor regression");
          return;
        }
      }
      if (action.contains("invocations") &&
          bridge.lastState().value("invocations").toString() !=
              action.value("invocations").toString()) {
        fail("unexpected guest execution count");
        return;
      }
      if (action.contains("error") &&
          !bridge.error().contains(action.value("error").toString())) {
        fail("expected diagnostic missing");
        return;
      }
      if (action.contains("ready") &&
          bridge.ready() != action.value("ready").toBool()) {
        fail("unexpected ready state");
        return;
      }
      if (action.contains("rooms") &&
          bridge.rooms().size() != action.value("rooms").toInt()) {
        fail("room count mismatch");
        return;
      }
      if (action.contains("pinned")) {
        bool pinned = false;
        for (const auto &r : bridge.rooms())
          if (r.toMap().value("id") == bridge.selectedRoom())
            pinned = r.toMap().value("pinned").toBool();
        if (pinned != action.value("pinned").toBool()) {
          fail("pin state mismatch");
          return;
        }
      }
      if (action.contains("accessibleError")) {
        m_shell.decorateFields();
        auto *item =
            find(m_shell.canvasWindow()->contentItem(),
                 "input_" + action.value("accessibleError").toString());
        auto *accessible =
            item ? QAccessible::queryAccessibleInterface(item) : nullptr;
        if (!accessible ||
            accessible->text(QAccessible::Description).isEmpty()) {
          fail("field accessible diagnostic missing");
          return;
        }
      }
    } else if (type == "settle") {
      if (!wait([&] { return !bridge.busy() && !bridge.switching(); })) {
        fail("settle timeout");
        return;
      }
    } else if (type == "quit") {
      sample(type);
      const auto directory = QFileInfo(m_report).absolutePath();
      QDir().mkpath(directory);
      m_shell.grab().save(directory + "/shell.png");
      const auto canvas = m_shell.canvasWindow()->grabWindow();
      if (!canvas.isNull())
        canvas.save(directory + "/canvas.png");
      QTest::mouseClick(m_shell.findChild<QPushButton *>("quitWorkbench"),
                        Qt::LeftButton);
      return;
    } else {
      fail("unknown action");
      return;
    }
    QTest::qWait(30);
    m_shell.decorateFields();
    sample(type);
  }
  sample("complete");
  m_shell.close();
}
void ConformanceDriver::finish(int exitCode) {
  sample("exit");
  QFile binary(QCoreApplication::applicationFilePath());
  if (!binary.open(QIODevice::ReadOnly))
    m_failure = "binary evidence unavailable";
  QJsonObject report{
      {"pid", QCoreApplication::applicationPid()},
      {"workerProcessPid", m_shell.bridge().workerPid()},
      {"exitCode", exitCode},
      {"qtVersion", qVersion()},
      {"platform", QGuiApplication::platformName()},
      {"binarySha256", QString(QCryptographicHash::hash(
                                   binary.readAll(), QCryptographicHash::Sha256)
                                   .toHex())},
      {"failure", m_failure},
      {"samples", m_samples},
      {"finalState", m_shell.bridge().lastState()},
      {"guiThread", m_shell.bridge().callbacksOnGuiThread()}};
  const auto directory = QFileInfo(m_report).absolutePath();
  QDir().mkpath(directory);
  if (!QFileInfo::exists(directory + "/shell.png"))
    m_shell.grab().save(directory + "/shell.png");
  QFile out(m_report);
  if (out.open(QIODevice::WriteOnly))
    out.write(QJsonDocument(report).toJson());
}
