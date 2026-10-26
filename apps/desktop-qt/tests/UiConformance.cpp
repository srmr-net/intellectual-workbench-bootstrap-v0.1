#include "UiBridge.h"
#include <QAccessible>
#include <QAccessibleInterface>
#include <QDir>
#include <QFile>
#include <QGuiApplication>
#include <QJsonArray>
#include <QJsonDocument>
#include <QQmlApplicationEngine>
#include <QQmlContext>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTemporaryDir>
#include <QtTest>

static QQuickItem *findItem(QQuickItem *item, const QString &name) {
  if (item->objectName() == name)
    return item;
  for (auto *child : item->childItems())
    if (auto *found = findItem(child, name))
      return found;
  return nullptr;
}
static QString root() { return qEnvironmentVariable("WB_REPOSITORY_ROOT"); }
static LaunchConfig config(const QString &store,
                           const QString &room = "qt-room",
                           bool restore = false) {
  return {root() + "/target/debug/wb-ui-worker",
          root() + "/target/ui-fixtures/mortgage-ui",
          store,
          qEnvironmentVariable("REGISTRY_PYTHON"),
          root() + "/tools/registry_validate.py",
          root() + "/target/debug/wb-registry-inspect",
          room,
          restore};
}
struct View {
  UiBridge bridge;
  QQmlApplicationEngine engine;
  QQuickWindow *window = nullptr;
  View(const LaunchConfig &c, int capacity = 128) : bridge(nullptr, capacity) {
    engine.rootContext()->setContextProperty("bridge", &bridge);
    engine.load(QUrl("qrc:/qml/Main.qml"));
    if (!engine.rootObjects().isEmpty())
      window = qobject_cast<QQuickWindow *>(engine.rootObjects().first());
    bridge.start(c);
  }
  QQuickItem *item(const QString &id) {
    return window ? findItem(window->contentItem(), id) : nullptr;
  }
};
class UiConformance : public QObject {
  Q_OBJECT
private slots:
  void initTestCase() {
    QQuickStyle::setStyle("Basic");
    QVERIFY(!root().isEmpty());
    QVERIFY(QFile::exists(root() + "/target/debug/wb-ui-worker"));
  }
  void ui_009_011_generic_form_accessibility_keyboard_and_theme() {
    QTemporaryDir dir;
    View view(config(dir.path() + "/registry"));
    QVERIFY(view.window);
    QTRY_VERIFY_WITH_TIMEOUT(view.bridge.ready(), 60000);
    QCOMPARE(view.bridge.controls().size(), 4);
    QTRY_VERIFY(view.item("input_principal"));
    const QStringList ids{"principal", "rate", "term"};
    const QStringList labels{"Principal", "Annual interest rate",
                             "Term (months)"};
    QJsonArray accessibility;
    for (int n = 0; n < ids.size(); ++n) {
      auto *item = view.item("input_" + ids[n]);
      QVERIFY(item);
      auto *a = QAccessible::queryAccessibleInterface(item);
      QVERIFY(a);
      QCOMPARE(a->role(), QAccessible::EditableText);
      QCOMPARE(a->text(QAccessible::Name), labels[n]);
      QVERIFY(a->state().focusable);
      accessibility.append(
          QJsonObject{{"id", ids[n]},
                      {"name", a->text(QAccessible::Name)},
                      {"role", int(a->role())},
                      {"focusable", bool(a->state().focusable)}});
    }
    auto *readout = view.item("output_payment");
    QVERIFY(readout);
    auto *a = QAccessible::queryAccessibleInterface(readout);
    QVERIFY(a);
    QCOMPARE(a->role(), QAccessible::StaticText);
    QCOMPARE(a->text(QAccessible::Name), QString("Monthly repayment"));
    auto *principal = view.item("input_principal");
    principal->forceActiveFocus();
    QTRY_COMPARE(view.window->activeFocusItem(), principal);
    for (char c : QByteArray("250000.00")) {
      QTest::keyClick(view.window, c);
    }
    QTest::keyClick(view.window, Qt::Key_Tab);
    QTRY_COMPARE(view.window->activeFocusItem(), view.item("input_rate"));
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    for (char c : QByteArray("5.00")) {
      QTest::keyClick(view.window, c);
    }
    QTest::keyClick(view.window, Qt::Key_Tab);
    QTRY_COMPARE(view.window->activeFocusItem(), view.item("input_term"));
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    for (char c : QByteArray("300")) {
      QTest::keyClick(view.window, c);
    }
    QTest::keyClick(view.window, Qt::Key_Tab);
    QTRY_COMPARE_WITH_TIMEOUT(readout->property("text").toString(),
                              QString("1461.48"), 15000);
    principal->forceActiveFocus();
    principal->setProperty("cursorPosition", 3);
    auto *identity = principal;
    view.bridge.edit("rate", "0", view.bridge.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(readout->property("text").toString(),
                              QString("833.33"), 15000);
    QCOMPARE(view.item("input_principal"), identity);
    QCOMPARE(view.window->activeFocusItem(), principal);
    QCOMPARE(principal->property("cursorPosition").toInt(), 3);
    view.bridge.setTextScale(2);
    view.bridge.setDarkMode(true);
    QTRY_VERIFY(principal->height() > 40);
    QVERIFY(view.bridge.callbacksOnGuiThread());
    const QString output = qEnvironmentVariable("WB_UI_EVIDENCE_DIR");
    QVERIFY(QDir().mkpath(output));
    QTest::qWait(100);
    const auto image = view.window->grabWindow();
    QVERIFY(!image.isNull());
    QVERIFY(image.save(output + "/form-dark-large.png"));
    view.bridge.setTextScale(1);
    view.bridge.setDarkMode(false);
    QTest::qWait(100);
    QVERIFY(view.window->grabWindow().save(output + "/form-light.png"));
    QFile file(output + "/accessibility.json");
    QVERIFY(file.open(QIODevice::WriteOnly));
    file.write(QJsonDocument(accessibility).toJson());
  }
  void ui_012_014_real_worker_committed_outputs_and_gui_thread() {
    QTemporaryDir dir;
    View view(config(dir.path() + "/registry"));
    QVERIFY(view.window);
    QTRY_VERIFY_WITH_TIMEOUT(view.bridge.ready(), 60000);
    auto edit = [&](const QString &id, const QString &text) {
      view.bridge.edit(id, text, view.bridge.viewEpoch());
    };
    edit("principal", "250000.00");
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    edit("rate", "5.00");
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    edit("term", "300");
    QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                              QString("1461.48"), 15000);
    edit("principal", "120000.00");
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    edit("rate", "0");
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    edit("term", "240");
    QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                              QString("500.00"), 15000);
    QVERIFY(view.bridge.lastProjection().value("roomRevision").isString());
    QVERIFY(view.bridge.callbacksOnGuiThread());
    QCOMPARE(view.bridge.error(), QString());
  }
  void ui_015_016_017_fences_queue_room_switch_and_teardown() {
    QTemporaryDir dir;
    View view(config(dir.path() + "/registry", "room-a"), 2);
    QVERIFY(view.window);
    QTRY_VERIFY_WITH_TIMEOUT(view.bridge.ready(), 60000);
    const auto epoch = view.bridge.m_epoch;
    const auto old = view.bridge.lastProjection();
    const auto count = view.bridge.appliedCount();
    view.bridge.accept(QJsonObject{{"protocol", "0.1"},
                                   {"kind", "projection"},
                                   {"projection", old}},
                       epoch);
    QCOMPARE(view.bridge.appliedCount(), count);
    QVERIFY(view.bridge.rejectedCount() > 0);
    view.bridge.edit("principal", "1", view.bridge.viewEpoch());
    view.bridge.edit("rate", "1", view.bridge.viewEpoch());
    view.bridge.edit("term", "1", view.bridge.viewEpoch());
    QCOMPARE(view.bridge.error(), QString("QUEUE_FULL"));
    QVERIFY(view.bridge.pendingCount() <= 2);
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    view.bridge.apply(view.bridge.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                              QString("1.00"), 15000);
    // A retained, unsubmitted edit survives Save and an accepted
    // different-field edit. Only an explicit Apply acknowledges and clears that
    // draft.
    view.bridge.setLocalDraft("term", "2", view.bridge.viewEpoch());
    QSignalSpy draftSaved(&view.bridge, &UiBridge::saved);
    view.bridge.save();
    QTRY_COMPARE(draftSaved.count(), 1);
    QVERIFY(view.bridge.hasDraft("term"));
    view.bridge.edit("rate", "0", view.bridge.viewEpoch());
    QTRY_COMPARE(view.bridge.pendingCount(), 0);
    QCOMPARE(view.bridge.error(), QString());
    QVERIFY(view.bridge.hasDraft("term"));
    QCOMPARE(view.bridge.committedValue("payment"), QString("1.00"));
    view.bridge.apply(view.bridge.viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                              QString("0.50"), 15000);
    QVERIFY(!view.bridge.hasDraft("term"));
    const auto oldEpoch = view.bridge.viewEpoch();
    view.bridge.start(config(dir.path() + "/registry", "room-b"));
    QTRY_VERIFY_WITH_TIMEOUT(view.bridge.ready(), 60000);
    QCOMPARE(view.bridge.lastProjection()
                 .value("key")
                 .toObject()
                 .value("roomId")
                 .toString(),
             QString("room-b"));
    const auto before = view.bridge.appliedCount();
    view.bridge.accept(QJsonObject{{"protocol", "0.1"},
                                   {"kind", "projection"},
                                   {"projection", old}},
                       epoch);
    QCOMPARE(view.bridge.appliedCount(), before);
    view.bridge.edit("principal", "999", oldEpoch);
    QCOMPARE(view.bridge.pendingCount(), 0);
    QCOMPARE(view.bridge.committedValue("principal"), QString());
    QPointer<QProcess> child = view.bridge.m_process;
    view.bridge.stop();
    QTRY_VERIFY(child.isNull());
    QVERIFY(!view.bridge.ready());
    QVERIFY(view.bridge.callbacksOnGuiThread());
  }
  void ui_009_010_017_generic_metadata_two_rooms_and_injection() {
    QTemporaryDir dir;
    auto alternate = config(dir.path() + "/registry", "second");
    alternate.source = root() + "/target/ui-fixtures/alternate-ui";
    View first(config(dir.path() + "/registry", "first"));
    View second(alternate);
    QTRY_VERIFY_WITH_TIMEOUT(first.bridge.ready(), 60000);
    QTRY_VERIFY_WITH_TIMEOUT(second.bridge.ready(), 60000);
    QVERIFY(!second.item("input_principal"));
    QVERIFY(second.item("input_amount"));
    auto fill = [](View &v, QStringList ids, QStringList values) {
      for (int i = 0; i < ids.size(); ++i) {
        v.bridge.edit(ids[i], values[i], v.bridge.viewEpoch());
        QTRY_COMPARE(v.bridge.pendingCount(), 0);
      }
    };
    fill(first, {"principal", "rate", "term"}, {"120000.00", "0", "240"});
    fill(second, {"amount", "interest", "duration"},
         {"250000.00", "5.00", "300"});
    QTRY_COMPARE(first.item("output_payment")->property("text").toString(),
                 QString("500.00"));
    QTRY_COMPARE(second.item("output_result")->property("text").toString(),
                 QString("1461.48"));
    const auto before = first.bridge.appliedCount();
    first.bridge.accept(
        QJsonObject{{"protocol", "0.1"},
                    {"kind", "projection"},
                    {"projection", second.bridge.lastProjection()}},
        first.bridge.m_epoch);
    QCOMPARE(first.bridge.appliedCount(), before);
    QCOMPARE(first.bridge.committedValue("payment"), QString("500.00"));
    second.bridge.setTextScale(2);
    QTest::qWait(100);
    // Valid long metadata remains inside the viewport at the maximum text
    // scale.
    for (auto *item :
         second.window->contentItem()->findChildren<QQuickItem *>()) {
      if (item->property("text").toString().startsWith("Long accessible")) {
        QVERIFY(item->height() > 50);
        QVERIFY(item->width() <= second.window->width());
      }
    }
    first.bridge.stop();
    second.bridge.stop();
    auto bad = config(dir.path() + "/bad-store", "malicious");
    bad.source = root() + "/target/ui-fixtures/malicious-ui";
    View rejected(bad);
    QTRY_COMPARE_WITH_TIMEOUT(rejected.bridge.error(),
                              QString("UI_UNAVAILABLE"), 60000);
    QVERIFY(!rejected.bridge.ready());
    QVERIFY(rejected.bridge.controls().isEmpty());
  }
  void ui_018_quit_restore_exact_pin_and_values() {
    QTemporaryDir dir;
    const QString store = dir.path() + "/registry";
    {
      View view(config(store, "restore-room"));
      QTRY_VERIFY_WITH_TIMEOUT(view.bridge.ready(), 60000);
      view.bridge.edit("principal", "120000.00", view.bridge.viewEpoch());
      QTRY_COMPARE(view.bridge.pendingCount(), 0);
      view.bridge.edit("rate", "0", view.bridge.viewEpoch());
      QTRY_COMPARE(view.bridge.pendingCount(), 0);
      view.bridge.edit("term", "240", view.bridge.viewEpoch());
      QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                                QString("500.00"), 15000);
      QSignalSpy saved(&view.bridge, &UiBridge::saved);
      view.bridge.save();
      QTRY_COMPARE(saved.count(), 1);
      view.bridge.edit("term", "120", view.bridge.viewEpoch());
      QTRY_COMPARE_WITH_TIMEOUT(view.bridge.committedValue("payment"),
                                QString("1000.00"), 15000);
      view.bridge.stop();
    }
    {
      View restored(config(store, "restore-room", true));
      QTRY_VERIFY_WITH_TIMEOUT(restored.bridge.ready(), 60000);
      QCOMPARE(restored.bridge.committedValue("principal"),
               QString("120000.00"));
      QCOMPARE(restored.bridge.committedValue("payment"), QString("1000.00"));
      restored.bridge.stop();
    }
    QDir objects(store + "/objects");
    const auto entries = objects.entryList(QDir::Dirs | QDir::NoDotAndDotDot);
    QVERIFY(!entries.isEmpty());
    QVERIFY(QDir(objects.filePath(entries.first())).removeRecursively());
    View missing(config(store, "restore-room", true));
    QTRY_COMPARE_WITH_TIMEOUT(missing.bridge.error(), QString("UI_UNAVAILABLE"),
                              60000);
    QVERIFY(!missing.bridge.ready());
    QVERIFY(missing.bridge.controls().isEmpty());
  }
};
QTEST_MAIN(UiConformance)
#include "UiConformance.moc"
