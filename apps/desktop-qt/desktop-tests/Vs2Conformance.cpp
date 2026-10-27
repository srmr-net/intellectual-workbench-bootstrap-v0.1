#include "DesktopShell.h"
#include <QAccessible>
#include <QAccessibleInterface>
#include <QApplication>
#include <QDir>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QtTest>
static QString root() { return qEnvironmentVariable("WB_REPOSITORY_ROOT"); }
static QQuickItem *item(QQuickItem *i, const QString &name) {
  if (i->objectName() == name)
    return i;
  for (auto *c : i->childItems())
    if (auto *v = item(c, name))
      return v;
  return nullptr;
}
struct Fixture {
  QTemporaryDir directory;
  DesktopConfig config;
  bool installed = false;
  Fixture() {
    const QString r = root();
    config = {r + "/target/debug/wb-desktop-worker",
              directory.path() + "/registry",
              directory.path() + "/workspace",
              qEnvironmentVariable("REGISTRY_PYTHON"),
              r + "/tools/registry_validate.py",
              r + "/target/debug/wb-registry-inspect"};
    QProcess p;
    p.start(r + "/target/debug/wb-desktop-install",
            {config.registry, config.python, config.semantic, config.abi,
             r + "/target/ui-fixtures/mortgage-ui"});
    installed = p.waitForFinished(120000) && p.exitCode() == 0;
  }
};
static bool waitFor(std::function<bool()> predicate, int ms = 60000) {
  QElapsedTimer timer;
  timer.start();
  while (!predicate()) {
    if (timer.elapsed() > ms)
      return false;
    QTest::qWait(20);
  }
  return true;
}
static bool openRoom(DesktopShell &shell) {
  auto &b = shell.bridge();
  if (!waitFor([&] { return !b.installed().isEmpty(); }))
    return false;
  b.createRoom(QJsonObject::fromVariantMap(
      b.installed().first().toMap().value("pin").toMap()));
  return waitFor([&] { return b.ready() && !b.busy(); });
}
static bool fill(DesktopShell &shell) {
  auto &b = shell.bridge();
  for (const auto &pair : QList<QPair<QString, QString>>{
           {"principal", "250000.00"}, {"rate", "5.00"}, {"term", "300"}}) {
    b.edit(pair.first, pair.second, b.viewEpoch());
    if (!waitFor([&] { return !b.busy(); }))
      return false;
  }
  return waitFor([&] {
    auto *v = item(shell.canvasWindow()->contentItem(), "output_payment");
    return v && v->property("text") == "1461.48";
  });
}
static bool finish(DesktopShell &shell) {
  QSignalSpy closed(&shell, &DesktopShell::closed);
  shell.close();
  return waitFor([&] { return closed.count() == 1; }) &&
         closed.first().first().toBool();
}
class Vs2Conformance : public QObject {
  Q_OBJECT
private slots:
  void initTestCase() {
    QApplication::setQuitOnLastWindowClosed(false);
    QQuickStyle::setStyle("Basic");
    QVERIFY(!root().isEmpty());
  }
  void vs2_010_pending_edit_room_switch_and_stale_callback() {
    Fixture f;
    QVERIFY(f.installed);
    DesktopShell s(f.config);
    s.show();
    QVERIFY(openRoom(s));
    QVERIFY(fill(s));
    auto &b = s.bridge();
    const auto oldFrame = b.lastState();
    const auto oldEpoch = b.viewEpoch();
    const auto first = b.selectedRoom();
    const auto pin = QJsonObject::fromVariantMap(
        b.installed().first().toMap().value("pin").toMap());
    b.edit("principal", "120000.00", b.viewEpoch());
    b.createRoom(pin);
    QVERIFY(b.switching());
    QTRY_VERIFY_WITH_TIMEOUT(!b.switching() && !b.busy(), 60000);
    QVERIFY(b.ready());
    QVERIFY(b.selectedRoom() != first);
    QCOMPARE(b.committedValue("payment"), QString());
    const auto selected = b.selectedRoom();
    b.accept(oldFrame);
    b.edit("principal", "999999.00", oldEpoch);
    QCOMPARE(b.selectedRoom(), selected);
    QCOMPARE(b.committedValue("principal"), QString());
    QVERIFY(b.rejectedCount() >= 2);
    b.selectRoom(first);
    QTRY_VERIFY_WITH_TIMEOUT(!b.switching() && !b.busy(), 60000);
    QCOMPARE(b.committedValue("principal"), QString("120000.00"));
    QVERIFY(b.committedValue("payment") != "1461.48");
    QVERIFY(finish(s));
  }
  void vs2_011_real_bounded_queue_full_and_explicit_apply() {
    Fixture f;
    QVERIFY(f.installed);
    DesktopShell s(f.config, 2);
    s.show();
    QVERIFY(openRoom(s));
    QVERIFY(fill(s));
    auto &b = s.bridge();
    b.edit("principal", "120000.00", b.viewEpoch());
    b.edit("rate", "0", b.viewEpoch());
    b.edit("term", "240", b.viewEpoch());
    QCOMPARE(b.pendingCount(), 2);
    QCOMPARE(b.error(), QString("QUEUE_FULL"));
    QVERIFY(
        s.findChild<QLabel *>("recoveryStatus")->text().contains("QUEUE_FULL"));
    QVERIFY(b.hasDraft("term"));
    QTRY_COMPARE_WITH_TIMEOUT(b.pendingCount(), 0, 60000);
    QVERIFY(b.hasDraft("term"));
    b.apply(b.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(b.committedValue("payment"), QString("500.00"),
                              60000);
    QVERIFY(!b.hasDraft("term"));
    QTRY_COMPARE(item(s.canvasWindow()->contentItem(), "output_payment")
                     ->property("text")
                     .toString(),
                 QString("500.00"));
    QVERIFY(b.callbacksOnGuiThread());
    QVERIFY(finish(s));
  }
  void vs2_012_real_conflict_repair_never_implicitly_retries() {
    Fixture f;
    QVERIFY(f.installed);
    DesktopShell s(f.config);
    s.show();
    QVERIFY(openRoom(s));
    QVERIFY(fill(s));
    auto &b = s.bridge();
    b.edit("principal", "120000.00", b.viewEpoch());
    b.edit("rate", "0", b.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(b.pendingCount(), 0, 60000);
    QVERIFY(b.error().contains("STATE CONFLICT"));
    QVERIFY(b.hasDraft("rate"));
    QCOMPARE(b.lastState().value("invocations").toString(), QString("2"));
    QTest::qWait(100);
    QCOMPARE(b.lastState().value("invocations").toString(), QString("2"));
    b.apply(b.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(b.committedValue("payment"), QString("400.00"),
                              60000);
    QCOMPARE(b.lastState().value("invocations").toString(), QString("3"));
    QVERIFY(finish(s));
  }
  void vs2_014_quit_drains_rejected_and_error_replies() {
    Fixture f;
    QVERIFY(f.installed);
    {
      DesktopShell s(f.config);
      s.show();
      QVERIFY(openRoom(s));
      auto &b = s.bridge();
      b.edit("principal", "-1", b.viewEpoch());
      QSignalSpy closed(&s, &DesktopShell::closed);
      s.close();
      QTRY_COMPARE_WITH_TIMEOUT(closed.count(), 1, 60000);
      QVERIFY(closed.first().first().toBool());
      QCOMPARE(b.pendingCount(), 0);
    }
    {
      DesktopShell s(f.config);
      s.show();
      auto &b = s.bridge();
      QTRY_VERIFY_WITH_TIMEOUT(b.ready(), 60000);
      b.edit("principal", "250000.00", b.viewEpoch());
      QVERIFY(!b.m_pending.isEmpty());
      const QString token = b.m_pending.firstKey();
      QSignalSpy closed(&s, &DesktopShell::closed);
      s.close();
      // Adversarial transport-error fixture exercises deferred quit; actual
      // native process still performs its real bounded request/flush/exit
      // lifecycle.
      b.accept(QJsonObject{{"protocol", "desktop.1"},
                           {"kind", "backpressure"},
                           {"token", token},
                           {"code", "QUEUE_FULL"}});
      QTRY_COMPARE_WITH_TIMEOUT(closed.count(), 1, 60000);
      QVERIFY(closed.first().first().toBool());
      QCOMPARE(b.pendingCount(), 0);
    }
  }
  void vs2_015_field_specific_accessible_errors_and_gui_thread() {
    Fixture f;
    QVERIFY(f.installed);
    DesktopShell s(f.config);
    s.show();
    QVERIFY(openRoom(s));
    auto &b = s.bridge();
    b.edit("principal", "-1", b.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(b.pendingCount(), 0, 60000);
    s.decorateFields();
    auto *principal = item(s.canvasWindow()->contentItem(), "input_principal");
    auto *rate = item(s.canvasWindow()->contentItem(), "input_rate");
    auto *a = QAccessible::queryAccessibleInterface(principal);
    auto *other = QAccessible::queryAccessibleInterface(rate);
    QVERIFY(a);
    QVERIFY(other);
    QCOMPARE(a->role(), QAccessible::EditableText);
    QCOMPARE(a->text(QAccessible::Name), QString("Principal"));
    QVERIFY(a->text(QAccessible::Description).contains("Principal"));
    QVERIFY(other->text(QAccessible::Description).isEmpty());
    QCOMPARE(b.lastState().value("invocations").toString(), QString("0"));
    QVERIFY(b.callbacksOnGuiThread());
    QVERIFY(finish(s));
  }
  void vs2_021_stale_frame_does_not_retire_live_request() {
    Fixture f;
    QVERIFY(f.installed);
    DesktopShell s(f.config);
    s.show();
    QVERIFY(openRoom(s));
    QVERIFY(fill(s));
    auto &b = s.bridge();
    auto stale = b.lastState();
    b.edit("principal", "120000.00", b.viewEpoch());
    QCOMPARE(b.pendingCount(), 1);
    stale["token"] = b.m_pending.firstKey();
    b.accept(stale);
    QCOMPARE(b.pendingCount(), 1);
    QCOMPARE(b.committedValue("payment"), QString("1461.48"));
    QTRY_COMPARE_WITH_TIMEOUT(b.pendingCount(), 0, 60000);
    QCOMPARE(b.committedValue("principal"), QString("120000.00"));
    const auto before = b.lastState();
    auto unknown = before;
    unknown["kind"] = "arbitrary";
    b.accept(unknown);
    QCOMPARE(b.lastState(), before);
    quint64 n = 0;
    QVERIFY(DesktopBridge::counter(QJsonValue("18446744073709551615"), n));
    QCOMPARE(n, std::numeric_limits<quint64>::max());
    QVERIFY(!DesktopBridge::counter(QJsonValue(1.0), n));
    QVERIFY(finish(s));
  }
};
QTEST_MAIN(Vs2Conformance)
#include "Vs2Conformance.moc"
