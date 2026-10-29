#include "ChatWindow.h"
#include <QAccessible>
#include <QCheckBox>
#include <QDialog>
#include <QDialogButtonBox>
#include <QDir>
#include <QJsonArray>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QTextBrowser>
#include <QTimer>
#include <QtTest>
#include <private/qguiapplication_p.h>
#include <qpa/qplatformaccessibility.h>
#include <qpa/qplatformintegration.h>
namespace {
int alerts = 0;
QAccessible::UpdateHandler prior = nullptr;
void observed(QAccessibleEvent *event) {
  if (event->type() == QAccessible::Alert && event->object() &&
      event->object()->objectName() == "chatStatus")
    ++alerts;
  if (prior)
    prior(event);
}
} // namespace
class ChatTests : public QObject {
  Q_OBJECT
private:
  QString worker() { return qEnvironmentVariable("WB_CHAT_TEST_WORKER"); }
  template <class T> T *get(ChatWindow &w, const char *name) {
    return w.findChild<T *>(name);
  }
  void enable(ChatWindow &w) {
    QTimer::singleShot(10, &w, [&w] {
      auto *dialog = w.findChild<QDialog *>("chatSettingsDialog");
      QVERIFY(dialog);
      dialog->findChild<QCheckBox *>("chatProviderEnabled")->setChecked(true);
      dialog->findChild<QLineEdit *>("chatCredentialSource")->clear();
      QTest::mouseClick(
          dialog->findChild<QDialogButtonBox *>("chatSettingsButtons")
              ->button(QDialogButtonBox::Save),
          Qt::LeftButton);
    });
    QTest::mouseClick(get<QPushButton>(w, "chatSettings"), Qt::LeftButton);
    QTRY_VERIFY(w.state().value("enabled").toBool());
  }
  void finish(ChatWindow &w) {
    w.close();
    QTRY_VERIFY(!w.isVisible());
  }
private slots:
  void initTestCase() {
    QVERIFY(!worker().isEmpty());
    qunsetenv("WB_LLM_SECRET_FILE");
    qunsetenv("OPENROUTER_API_KEY");
    qunsetenv("OPENAI_API_KEY");
    if (auto *platform = QGuiApplicationPrivate::platformIntegration())
      if (auto *a = platform->accessibility())
        a->setActive(true);
    QAccessible::setActive(true);
    prior = QAccessible::installUpdateHandler(observed);
    QVERIFY(QAccessible::isActive());
  }
  void cleanupTestCase() { QAccessible::installUpdateHandler(prior); }
  void contextual_chat_enter_and_inert_text() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    QCOMPARE(w.state().value("calls").toInt(), 0);
    QVERIFY(!get<QPushButton>(w, "chatSend")->isEnabled());
    enable(w);
    auto *e = get<QPlainTextEdit>(w, "chatComposer");
    const int initialAlerts = alerts;
    e->setPlainText("Remember <b>blue</b>.");
    e->setFocus();
    QTest::keyClick(e, Qt::Key_Return);
    QTRY_COMPARE(w.state().value("messages").toArray().size(), 2);
    e->setPlainText("What did I say?");
    QTest::keyClick(e, Qt::Key_Return);
    QTRY_COMPARE(w.state().value("messages").toArray().size(), 4);
    QCOMPARE(w.state().value("calls").toInt(), 2);
    QVERIFY(alerts > initialAlerts);
    auto *accessible =
        QAccessible::queryAccessibleInterface(get<QLabel>(w, "chatStatus"));
    QVERIFY(accessible);
    QCOMPARE(accessible->text(QAccessible::Name),
             get<QLabel>(w, "chatStatus")->text());
    QCOMPARE(accessible->text(QAccessible::Description),
             get<QLabel>(w, "chatStatus")->text());
    auto *transcript = get<QTextBrowser>(w, "chatTranscript");
    QVERIFY(transcript->toPlainText().contains(
        "You previously said: Remember <b>blue</b>."));
    QVERIFY(!transcript->openExternalLinks());
    QVERIFY(!transcript->openLinks());
    const auto evidence = qEnvironmentVariable("WB_CHAT_EVIDENCE_DIR");
    if (!evidence.isEmpty()) {
      QDir().mkpath(evidence);
      w.grab().save(evidence + "/conversation.png");
    }
    QVERIFY(get<QLabel>(w, "chatProvider")->text().contains("fixture"));
    finish(w);
  }
  void shift_enter_and_draft_preservation() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    enable(w);
    auto *e = get<QPlainTextEdit>(w, "chatComposer");
    e->setPlainText("First line");
    e->moveCursor(QTextCursor::End);
    QTest::keyClick(e, Qt::Key_Return, Qt::ShiftModifier);
    QVERIFY(e->toPlainText().contains('\n'));
    QCOMPARE(w.state().value("calls").toInt(), 0);
    finish(w);
  }
  void new_chat_and_switch_do_not_send_or_leak() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    enable(w);
    auto *e = get<QPlainTextEdit>(w, "chatComposer");
    e->setPlainText("First conversation private text");
    QTest::mouseClick(get<QPushButton>(w, "chatSend"), Qt::LeftButton);
    QTRY_COMPARE(w.state().value("messages").toArray().size(), 2);
    QTest::mouseClick(get<QPushButton>(w, "chatNew"), Qt::LeftButton);
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-2"));
    QCOMPARE(w.state().value("messages").toArray().size(), 0);
    QCOMPARE(w.state().value("calls").toInt(), 1);
    QVERIFY(!get<QTextBrowser>(w, "chatTranscript")
                 ->toPlainText()
                 .contains("private text"));
    auto *list = get<QListWidget>(w, "chatSessions");
    QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                      list->visualItemRect(list->item(0)).center());
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-1"));
    QVERIFY(get<QTextBrowser>(w, "chatTranscript")
                ->toPlainText()
                .contains("private text"));
    QCOMPARE(w.state().value("calls").toInt(), 1);
    finish(w);
  }
  void cancellation_and_missing_source_are_understandable() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    enable(w);
    get<QPlainTextEdit>(w, "chatComposer")->setPlainText("Stop this turn");
    QTest::mouseClick(get<QPushButton>(w, "chatSend"), Qt::LeftButton);
    QTest::mouseClick(get<QPushButton>(w, "chatStop"), Qt::LeftButton);
    QTRY_VERIFY(!w.state().value("busy").toBool());
    QTRY_COMPARE(w.state().value("messages").toArray().size(), 1);
    QCOMPARE(w.state()
                 .value("messages")
                 .toArray()[0]
                 .toObject()
                 .value("status")
                 .toString(),
             QString("cancelled"));
    QTimer::singleShot(10, &w, [&w] {
      auto *d = w.findChild<QDialog *>("chatSettingsDialog");
      d->findChild<QLineEdit *>("chatCredentialSource")
          ->setText("/nonexistent/private-chat-key");
      d->findChild<QDialogButtonBox *>("chatSettingsButtons")
          ->button(QDialogButtonBox::Save)
          ->click();
    });
    get<QPushButton>(w, "chatSettings")->click();
    QTRY_VERIFY(!w.state().value("credentialAvailable").toBool());
    QVERIFY(get<QLabel>(w, "chatProvider")->text().contains("fixture"));
    QVERIFY(!get<QPushButton>(w, "chatSend")->isEnabled());
    finish(w);
  }
  void failed_retry_is_bound_to_selected_conversation() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    enable(w);
    auto *composer = get<QPlainTextEdit>(w, "chatComposer");
    composer->setPlainText("fixture:rate-limit");
    get<QPushButton>(w, "chatSend")->click();
    QTRY_VERIFY(get<QPushButton>(w, "chatRetry")->isVisible());
    get<QPushButton>(w, "chatNew")->click();
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-2"));
    QVERIFY(!get<QPushButton>(w, "chatRetry")->isVisible());
    get<QPushButton>(w, "chatRetry")->click();
    QVERIFY(!composer->toPlainText().contains("fixture:rate-limit"));
    auto *list = get<QListWidget>(w, "chatSessions");
    QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                      list->visualItemRect(list->item(0)).center());
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-1"));
    QVERIFY(get<QPushButton>(w, "chatRetry")->isVisible());
    get<QPushButton>(w, "chatRetry")->click();
    QCOMPARE(composer->toPlainText(), QString("fixture:rate-limit"));
    finish(w);
  }
  void switching_fences_drafts_and_oversize_has_feedback() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    enable(w);
    auto *composer = get<QPlainTextEdit>(w, "chatComposer");
    composer->setPlainText("Draft belonging to first chat");
    get<QPushButton>(w, "chatNew")->click();
    QVERIFY(composer->isReadOnly());
    QTest::keyClicks(composer, "must not migrate");
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-2"));
    QVERIFY(composer->toPlainText().isEmpty());
    QVERIFY(!composer->isReadOnly());
    composer->setPlainText(QString(4097, 'x'));
    QVERIFY(get<QLabel>(w, "chatMessageLimit")->isVisible());
    QVERIFY(!get<QPushButton>(w, "chatSend")->isEnabled());
    auto *list = get<QListWidget>(w, "chatSessions");
    QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                      list->visualItemRect(list->item(0)).center());
    QTRY_COMPARE(w.state().value("selected").toString(), QString("chat-1"));
    QCOMPARE(composer->toPlainText(), QString("Draft belonging to first chat"));
    QVERIFY(!get<QLabel>(w, "chatMessageLimit")->isVisible());
    finish(w);
  }
  void settings_cancel_and_keyboard_focus() {
    ChatWindow w(worker(), {}, true);
    w.show();
    QTRY_VERIFY(!w.state().value("selected").toString().isEmpty());
    QTimer::singleShot(10, &w, [&w] {
      auto *d = w.findChild<QDialog *>("chatSettingsDialog");
      d->findChild<QCheckBox *>("chatProviderEnabled")->setChecked(true);
      QTest::keyClick(d, Qt::Key_Escape);
    });
    get<QPushButton>(w, "chatSettings")->click();
    QVERIFY(!w.state().value("enabled").toBool());
    QCOMPARE(w.state().value("calls").toInt(), 0);
    finish(w);
  }
};
QTEST_MAIN(ChatTests)
#include "tests.moc"
