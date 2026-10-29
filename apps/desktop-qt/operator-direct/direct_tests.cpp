#include "DirectPanel.h"
#include <QApplication>
#include <QFile>
#include <QInputMethodEvent>
#include <QJsonArray>
#include <QJsonObject>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QQuickStyle>
#include <QTemporaryDir>
#include <QtTest>
#include <sys/stat.h>
class DirectTests : public QObject {
  Q_OBJECT
private slots:
  void request_start_and_local_recalculation_data() {
    QTest::addColumn<bool>("live");
    QTest::newRow("actual-adapter-mocked-http") << true;
    QTest::newRow("offline-without-credentials") << false;
  }
  void request_start_and_local_recalculation() {
    QFETCH(bool, live);
    QTemporaryDir temp;
    QVERIFY(temp.isValid());
    const auto root = qEnvironmentVariable("WB_REPOSITORY_ROOT");
    QVERIFY(!root.isEmpty());
    const auto python = qEnvironmentVariable("REGISTRY_PYTHON");
    const auto native = root + "/target/debug/";
    QProcess install;
    install.start(native + "wb-desktop-install",
                  {temp.path() + "/registry", python,
                   root + "/tools/registry_validate.py",
                   native + "wb-registry-inspect",
                   root + "/target/ui-fixtures/mortgage-ui"});
    QVERIFY(install.waitForFinished(90000));
    QCOMPARE(install.exitCode(), 0);
    const auto key = temp.path() + "/key";
    QFile secret(key);
    QVERIFY(secret.open(QIODevice::WriteOnly));
    secret.write("direct-operator-synthetic-key");
    secret.close();
    QVERIFY(::chmod(key.toLocal8Bit().constData(), 0600) == 0);
    if (live)
      qputenv("WB_LLM_SECRET_FILE", key.toUtf8());
    else
      qunsetenv("WB_LLM_SECRET_FILE");
    if (live)
      qputenv("WB_VS4_TEST_MODE", "success");
    else
      qunsetenv("WB_VS4_TEST_MODE");
    const auto capture = temp.path() + "/capture.jsonl";
    qputenv("WB_VS4_TEST_CAPTURE", capture.toUtf8());
    qunsetenv("OPENROUTER_API_KEY");
    qunsetenv("WB_LLM_BUDGET");
    qunsetenv("WB_LLM_MODEL");
    qunsetenv("WB_LLM_PROVIDER");
    DesktopShell shell({native + "wb-vslice004-worker",
                        temp.path() + "/registry", temp.path() + "/workspace",
                        python, root + "/tools/registry_validate.py",
                        native + "wb-registry-inspect"});
    DirectPanel panel(shell, live);
    shell.show();
    auto *ask = panel.findChild<QPushButton *>("directAsk");
    auto *start = panel.findChild<QPushButton *>("directStart");
    auto *editor = panel.findChild<QPlainTextEdit *>("directRequest");
    QVERIFY(ask && start && editor);
    QTRY_VERIFY_WITH_TIMEOUT(ask->isEnabled(), 60000);
    editor->setFocus();
    QInputMethodEvent input;
    input.setCommitString(
        QString::fromUtf8("I need a mortgage calculator for a £250,000 loan "
                          "over 25 years at 5%."));
    QApplication::sendEvent(editor, &input);
    QTest::mouseClick(ask, Qt::LeftButton);
    QTRY_VERIFY_WITH_TIMEOUT(start->isEnabled(), 60000);
    QCOMPARE(shell.bridge().lastState().value("rooms").toArray().size(), 0);
    QCOMPARE(panel.providerState().value("remoteAttempts").toString(),
             QString(live ? "1" : "0"));
    QTest::mouseClick(start, Qt::LeftButton);
    QTRY_COMPARE_WITH_TIMEOUT(shell.bridge().committedValue("payment"),
                              QString("1461.48"), 60000);
    QCOMPARE(shell.bridge().lastState().value("rooms").toArray().size(), 1);
    shell.bridge().edit("principal", "300000", shell.bridge().viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(shell.bridge().committedValue("payment"),
                              QString("1753.77"), 60000);
    QFile calls(capture);
    if (live) {
      QVERIFY(calls.open(QIODevice::ReadOnly));
      QCOMPARE(calls.readAll().count('\n'), 1);
    } else
      QVERIFY(!calls.exists());
    auto *process = shell.bridge().findChild<QProcess *>();
    shell.close();
    QTRY_VERIFY_WITH_TIMEOUT(process->state() == QProcess::NotRunning, 15000);
  }
};
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  DirectTests tests;
  return QTest::qExec(&tests, argc, argv);
}
#include "direct_tests.moc"
