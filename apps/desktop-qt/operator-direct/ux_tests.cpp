#include "DirectPanel.h"
#include "TraceRecorder.h"
#include <QAccessible>
#include <QApplication>
#include <QCheckBox>
#include <QCryptographicHash>
#include <QDialog>
#include <QDialogButtonBox>
#include <QFile>
#include <QGroupBox>
#include <QInputMethodEvent>
#include <QJsonArray>
#include <QJsonDocument>
#include <QLabel>
#include <QLineEdit>
#include <QListWidget>
#include <QMetaMethod>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QQuickItem>
#include <QQuickStyle>
#include <QQuickWindow>
#include <QTabBar>
#include <QTabWidget>
#include <QTemporaryDir>
#include <QtTest>
#include <private/qguiapplication_p.h>
#include <qpa/qplatformaccessibility.h>
#include <qpa/qplatformintegration.h>
#include <sys/stat.h>
#include <unistd.h>
namespace {
QString root, python, out;
const QString intent = QString::fromUtf8(
    "I need a mortgage calculator for a £250,000 loan over 25 years at 5%.");
QByteArray fakeToken() {
  return QStringList({"ux", "001", "private", "synthetic", "token"})
      .join("-")
      .toUtf8();
}
QQuickItem *item(QQuickItem *root, const QString &name) {
  if (root->objectName() == name)
    return root;
  for (auto *child : root->childItems())
    if (auto *found = item(child, name))
      return found;
  return nullptr;
}
int alerts = 0;
QAccessible::UpdateHandler previous = nullptr;
void alert(QAccessibleEvent *e) {
  if (e->type() == QAccessible::Alert && e->object() &&
      (e->object()->objectName() == "directStatus" ||
       e->object()->objectName() == "workspaceStatus"))
    ++alerts;
  if (previous)
    previous(e);
}
bool wait(const std::function<bool()> &p, int limit = 60000) {
  QElapsedTimer t;
  t.start();
  while (!p()) {
    if (t.elapsed() > limit)
      return false;
    QTest::qWait(10);
  }
  return true;
}
struct Fixture {
  QTemporaryDir temp;
  std::unique_ptr<DesktopShell> shell;
  std::unique_ptr<DirectPanel> panel;
  QString registry, workspace, capture;
  bool install(QString package = {}) {
    registry = temp.path() + "/registry";
    workspace = temp.path() + "/workspace";
    capture = temp.path() + "/calls.jsonl";
    if (package.isEmpty())
      package = root + "/target/ui-fixtures/mortgage-ui";
    QProcess p;
    p.start(root + "/target/debug/wb-desktop-install",
            {registry, python, root + "/tools/registry_validate.py",
             root + "/target/debug/wb-registry-inspect", package});
    return p.waitForFinished(90000) && p.exitCode() == 0;
  }
  bool boot(bool live = false, QString mode = "success", bool secret = true) {
    const auto key = temp.path() + "/private-source";
    QFile f(key);
    if (!f.open(QIODevice::WriteOnly))
      return false;
    f.write(fakeToken());
    f.close();
    ::chmod(key.toLocal8Bit().constData(), 0600);
    for (auto name : {"OPENROUTER_API_KEY", "OPENAI_API_KEY", "WB_LLM_MODEL",
                      "WB_LLM_BUDGET", "WB_LLM_PROVIDER"})
      qunsetenv(name);
    if (secret)
      qputenv("WB_LLM_SECRET_FILE", key.toUtf8());
    else
      qunsetenv("WB_LLM_SECRET_FILE");
    if (live)
      qputenv("WB_VS4_TEST_MODE", mode.toUtf8());
    else
      qunsetenv("WB_VS4_TEST_MODE");
    qputenv("WB_VS4_TEST_CAPTURE", capture.toUtf8());
    return open(live);
  }
  bool open(bool live) {
    shell = std::make_unique<DesktopShell>(
        DesktopConfig{root + "/target/debug/wb-vslice004-worker", registry,
                      workspace, python, root + "/tools/registry_validate.py",
                      root + "/target/debug/wb-registry-inspect"});
    panel = std::make_unique<DirectPanel>(*shell, live);
    shell->show();
    return wait([&] {
      return !panel->busy() && !shell->bridge().lastState().isEmpty();
    });
  }
  template <class T> T *get(const char *name) {
    return shell->findChild<T *>(name);
  }
  bool ask(QString text = intent) {
    auto *e = get<QPlainTextEdit>("directRequest");
    QApplication::setActiveWindow(shell.get());
    e->setFocus();
    QTest::qWait(20);
    if (!wait([&] { return !shell->bridge().busy(); }))
      return false;
    QTest::keyClick(e, Qt::Key_A, Qt::ControlModifier);
    QInputMethodEvent event;
    event.setCommitString(text);
    QApplication::sendEvent(e, &event);
    QTest::mouseClick(get<QPushButton>("directAsk"), Qt::LeftButton);
    return wait([&] { return !panel->busy(); });
  }
  bool start() {
    QTest::mouseClick(get<QPushButton>("directStart"), Qt::LeftButton);
    return wait([&] {
      return !panel->busy() && !shell->bridge().busy() &&
             !shell->bridge().switching();
    });
  }
  bool close() {
    if (!shell)
      return true;
    auto *p = shell->bridge().findChild<QProcess *>();
    shell->close();
    return wait([&] { return p->state() == QProcess::NotRunning; }, 15000);
  }
  void shot(QString name) {
    QDir().mkpath(out);
    shell->grab().save(out + "/" + name + "-workspace.png");
    shell->canvasWindow()->grabWindow().save(out + "/" + name + "-canvas.png");
  }
  ~Fixture() {
    if (shell) {
      close();
      panel.reset();
      shell.reset();
    }
  }
};
void save(QString file, QJsonObject value) {
  QFile f(out + "/" + file);
  if (f.open(QIODevice::WriteOnly))
    f.write(QJsonDocument(value).toJson());
}
} // namespace
class UxTests : public QObject {
  Q_OBJECT
private slots:
  void initTestCase() {
    root = qEnvironmentVariable("WB_REPOSITORY_ROOT");
    python = qEnvironmentVariable("REGISTRY_PYTHON");
    out = qEnvironmentVariable("WB_UX_EVIDENCE_DIR",
                               root + "/target/ux-001-evidence");
    QVERIFY(!root.isEmpty() && !python.isEmpty());
    QDir().mkpath(out);
    if (auto *platform = QGuiApplicationPrivate::platformIntegration())
      if (auto *a = platform->accessibility())
        a->setActive(true);
    QAccessible::setActive(true);
    previous = QAccessible::installUpdateHandler(alert);
    QVERIFY(QAccessible::isActive());
  }
  void ux_001_default_navigation_and_readonly_components() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot());
    QVERIFY(f.get<QLabel>("workspaceEmptyCanvas")->isVisible());
    QVERIFY(!f.get<QListWidget>("installedComponents")->isVisible());
    QVERIFY(!f.get<QPushButton>("openComponent")->isVisible());
    QVERIFY(!f.get<QLabel>("selectedIdentity")->isVisible());
    QVERIFY(!f.get<QPlainTextEdit>("operatorDiagnostics")->isVisible());
    QVERIFY(f.get<QListWidget>("pinnedRooms")->isVisible());
    QVERIFY(f.get<QListWidget>("workspacePinnedTools")->isVisible());
    QVERIFY(f.get<QListWidget>("workspaceRecentActivity")->isVisible());
    QVERIFY(!f.get<QLabel>("workspaceStatus")->text().contains("attention"));
    f.shot("default");
    auto *dialog = f.get<QDialog>("workbenchSettings");
    bool inspected = false;
    auto *editor = f.get<QPlainTextEdit>("directRequest");
    QApplication::setActiveWindow(f.shell.get());
    editor->setFocus();
    QTimer::singleShot(50, dialog, [&] {
      auto *tabs = dialog->findChild<QTabWidget *>("settingsSections");
      QTest::keyClick(tabs, Qt::Key_Tab, Qt::ControlModifier);
      auto *list = f.get<QListWidget>("installedComponents");
      inspected =
          dialog->isVisible() && tabs->currentIndex() == 1 &&
          list->isVisible() && list->count() == 1 &&
          dialog->findChild<QLineEdit *>("providerModel")->isReadOnly() &&
          dialog->findChild<QLineEdit *>("providerEndpoint")->isReadOnly();
      const auto pin = list->item(0)->data(Qt::UserRole).toMap();
      inspected = inspected &&
                  f.get<QPlainTextEdit>("componentInformation")
                      ->toPlainText()
                      .contains(pin.value("sha256").toString()) &&
                  f.get<QPlainTextEdit>("componentInformation")
                      ->toPlainText()
                      .contains(f.registry);
      QTest::mouseDClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                         list->visualItemRect(list->item(0)).center());
      QTest::qWait(50);
      inspected = inspected && f.shell->bridge().rooms().isEmpty();
      dialog->grab().save(out + "/components-settings.png");
      QTest::keyClick(dialog, Qt::Key_Escape);
    });
    QTimer::singleShot(3000, dialog, &QDialog::reject);
    QTest::keyClick(editor, Qt::Key_Comma, Qt::ControlModifier);
    QVERIFY(inspected);
    QVERIFY(!dialog->isVisible());
    QTRY_VERIFY(editor->hasFocus());
    QVERIFY(!f.get<QListWidget>("installedComponents")->isVisible());
    QVERIFY(f.shell->bridge().rooms().isEmpty());
    QVERIFY(!QFile::exists(f.capture));
  }
  void ux_002_session_settings_and_cancelled_changes() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot(false));
    auto *dialog = f.get<QDialog>("workbenchSettings");
    bool changed = false;
    QTimer::singleShot(30, dialog, [&] {
      auto *enabled = dialog->findChild<QCheckBox *>("providerEnabled");
      enabled->setFocus();
      QTest::keyClick(enabled, Qt::Key_Space);
      changed = enabled->isChecked();
      QTest::mouseClick(dialog->findChild<QDialogButtonBox *>("settingsButtons")
                            ->button(QDialogButtonBox::Save),
                        Qt::LeftButton);
    });
    QTest::mouseClick(f.get<QPushButton>("directSettings"), Qt::LeftButton);
    QVERIFY(changed);
    QTRY_VERIFY(!f.panel->busy());
    QCOMPARE(f.panel->providerState()
                 .value("profile")
                 .toObject()
                 .value("provider")
                 .toString(),
             QString("openrouter-chat"));
    QVERIFY(!QFile::exists(f.capture));
    QTimer::singleShot(30, dialog, [&] {
      auto *enabled = dialog->findChild<QCheckBox *>("providerEnabled");
      enabled->setChecked(false);
      QTest::keyClick(dialog, Qt::Key_Escape);
    });
    QTest::mouseClick(f.get<QPushButton>("directSettings"), Qt::LeftButton);
    QCOMPARE(f.panel->providerState()
                 .value("profile")
                 .toObject()
                 .value("provider")
                 .toString(),
             QString("openrouter-chat"));
    QVERIFY(f.get<QLineEdit>("providerCredentialSource")->text().isEmpty());
  }
  void ux_003_keyboard_ask_approval_and_reactive() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot(true));
    const auto initialAlerts = alerts;
    QVERIFY(f.ask());
    auto *summary = f.get<QLabel>("operatorSuggestionSummary");
    QVERIFY(summary->text().contains("Mortgage"));
    for (const auto &v : {"250000", "GBP", "5", "300", "Version"})
      QVERIFY(summary->text().contains(v));
    QVERIFY(f.shell->bridge().rooms().isEmpty());
    QVERIFY(f.get<QGroupBox>("operatorSuggestion")->isVisible());
    QCOMPARE(f.panel->providerState().value("remoteAttempts").toString(),
             QString("1"));
    QVERIFY(alerts > initialAlerts);
    auto *editor = f.get<QPlainTextEdit>("directRequest");
    QApplication::setActiveWindow(f.shell.get());
    editor->setFocus();
    QTest::keyClick(editor, Qt::Key_Tab);
    QCOMPARE(QApplication::focusWidget(), f.get<QPushButton>("directAsk"));
    QTest::keyClick(QApplication::focusWidget(), Qt::Key_Tab);
    QCOMPARE(QApplication::focusWidget(), f.get<QPushButton>("directStart"));
    QTest::keyClick(QApplication::focusWidget(), Qt::Key_Space);
    QTRY_COMPARE_WITH_TIMEOUT(f.shell->bridge().committedValue("payment"),
                              QString("1461.48"), 60000);
    QCOMPARE(f.shell->bridge().rooms().size(), 1);
    f.shot("started");
    auto *input =
        item(f.shell->canvasWindow()->contentItem(), "input_principal");
    QVERIFY(input);
    const auto invocations = f.shell->bridge().lastState().value("invocations");
    f.get<QWidget>("semanticCanvas")->setFocus(Qt::OtherFocusReason);
    f.shell->canvasWindow()->requestActivate();
    QTest::qWait(30);
    input->forceActiveFocus();
    QTRY_COMPARE(f.shell->canvasWindow()->activeFocusItem(), input);
    editor->setFocus();
    QTest::qWait(100);
    QVERIFY(!f.shell->bridge().busy());
    QCOMPARE(f.shell->bridge().lastState().value("invocations"), invocations);
    f.get<QWidget>("semanticCanvas")->setFocus(Qt::OtherFocusReason);
    f.shell->canvasWindow()->requestActivate();
    QTest::qWait(30);
    input->forceActiveFocus();
    auto *window = f.shell->canvasWindow();
    QTest::keyClick(window, Qt::Key_A, Qt::ControlModifier);
    QTRY_COMPARE(window->activeFocusItem(), input);
    for (char ch : QByteArray("300000"))
      QTest::keyClick(window, ch);
    QVERIFY(f.shell->bridge().hasDraft("principal"));
    QTest::keyClick(window, Qt::Key_Return);
    QTRY_COMPARE_WITH_TIMEOUT(f.shell->bridge().committedValue("payment"),
                              QString("1753.77"), 60000);
    QFile calls(f.capture);
    QVERIFY(calls.open(QIODevice::ReadOnly));
    QCOMPARE(calls.readAll().count('\n'), 1);
  }
  void ux_004_history_never_becomes_model_context() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot(true));
    QVERIFY(f.ask("PRIVATE-PRIOR-CONVERSATION"));
    QVERIFY(!f.get<QPushButton>("directStart")->isEnabled());
    QVERIFY(f.ask());
    QFile calls(f.capture);
    QVERIFY(calls.open(QIODevice::ReadOnly));
    const auto body = calls.readAll();
    QVERIFY(!body.contains("PRIVATE-PRIOR-CONVERSATION"));
    QVERIFY(!body.contains(fakeToken()));
    QCOMPARE(body.count('\n'), 1);
    QVERIFY(f.get<QPlainTextEdit>("operatorConversation")
                ->toPlainText()
                .contains("PRIVATE-PRIOR-CONVERSATION"));
    QVERIFY(f.start());
    QFile snapshot(f.workspace + "/workspace.json");
    QVERIFY(snapshot.open(QIODevice::ReadOnly));
    const auto bytes = snapshot.readAll();
    QVERIFY(!bytes.contains("PRIVATE-PRIOR-CONVERSATION"));
    QVERIFY(!bytes.contains(fakeToken()));
  }
  void ux_005_provider_errors_are_understandable() {
    {
      Fixture f;
      QVERIFY(f.install());
      QVERIFY(f.boot(true, "success", false));
      QVERIFY(!f.get<QPushButton>("directAsk")->isEnabled());
      QVERIFY(f.get<QLabel>("directStatus")->text().contains("credential"));
      QVERIFY(!QFile::exists(f.capture));
    }
    for (const auto &mode : {"candidate", "malformed", "auth", "refused"}) {
      Fixture f;
      QVERIFY(f.install());
      QVERIFY(f.boot(true, mode));
      QVERIFY(f.ask());
      QVERIFY(!f.get<QPushButton>("directStart")->isEnabled());
      QVERIFY(f.shell->bridge().rooms().isEmpty());
      const auto text = f.get<QLabel>("directStatus")->text();
      QVERIFY(!text.isEmpty());
      QVERIFY(!text.contains("SCHEMA_MISMATCH"));
      QVERIFY(!text.contains(fakeToken()));
    }
  }
  void ux_006_partial_initialization_and_draft_truthfulness() {
    Fixture f;
    QVERIFY(f.install(root + "/target/vs2-fixtures/adversary-ui"));
    QVERIFY(f.boot());
    QVERIFY(f.ask(QString::fromUtf8(
        "I need a mortgage calculator for a £5.00 loan over 25 years at 5%.")));
    QVERIFY(f.start());
    QCOMPARE(f.shell->bridge().rooms().size(), 1);
    const auto text =
        f.get<QPlainTextEdit>("operatorConversation")->toPlainText();
    QVERIFY(text.contains("initial calculation failed"));
    QVERIFY(!text.contains("workspace is unchanged"));
    QVERIFY(f.get<QLabel>("directStatus")->text().contains("workspace saved"));
    f.shell->bridge().edit("principal", "bad", f.shell->bridge().viewEpoch());
    QTRY_VERIFY(!f.shell->bridge().busy());
    QVERIFY(f.shell->bridge().hasAnyDraft());
    QVERIFY(!f.get<QLabel>("workspaceStatus")->text().contains("saved"));
    QVERIFY(f.get<QLabel>("workspaceStatus")->text().contains("Unapplied"));
  }
  void ux_007_pins_focus_and_cold_process() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot(true));
    QVERIFY(f.ask());
    QVERIFY(f.start());
    f.shell->bridge().edit("principal", "300000",
                           f.shell->bridge().viewEpoch());
    QTRY_COMPARE_WITH_TIMEOUT(f.shell->bridge().committedValue("payment"),
                              QString("1753.77"), 60000);
    QTest::mouseClick(f.get<QPushButton>("pinCanvas"), Qt::LeftButton);
    QTRY_COMPARE(f.get<QListWidget>("workspacePinnedTools")->count(), 1);
    auto *pins = f.get<QListWidget>("workspacePinnedTools");
    QApplication::setActiveWindow(f.shell.get());
    pins->setFocus();
    pins->setCurrentRow(0);
    const auto id = pins->currentItem()->data(Qt::UserRole).toString();
    f.shell->bridge().edit("principal", "bad", f.shell->bridge().viewEpoch());
    QTRY_VERIFY(!f.shell->bridge().busy());
    QVERIFY(pins->hasFocus());
    QVERIFY(pins->currentItem());
    QCOMPARE(pins->currentItem()->data(Qt::UserRole).toString(), id);
    f.shell->bridge().edit("principal", "300000",
                           f.shell->bridge().viewEpoch());
    QTRY_VERIFY(!f.shell->bridge().busy());
    const auto first = f.shell->bridge().lastState();
    const auto worker = f.shell->bridge().workerPid();
    QVERIFY(f.close());
    QFile snapshot(f.workspace + "/workspace.json");
    QVERIFY(snapshot.open(QIODevice::ReadOnly));
    const auto saved = snapshot.readAll();
    snapshot.close();
    QProcess cold;
    auto env = QProcessEnvironment::systemEnvironment();
    for (const auto &key : env.keys())
      if (key.startsWith("WB_LLM_") || key.startsWith("WB_VS4_") ||
          key.startsWith("OPENROUTER_") || key.startsWith("OPENAI_"))
        env.remove(key);
    env.insert("WB_OPERATOR_PROVIDER", "disabled");
    env.insert("WB_UX_COLD_REGISTRY", f.registry);
    env.insert("WB_UX_COLD_WORKSPACE", f.workspace);
    env.insert("WB_UX_COLD_REPORT", f.temp.path() + "/cold.json");
    cold.setProcessEnvironment(env);
    cold.start(root + "/target/debug/wb-desktop-offline",
               {QCoreApplication::applicationFilePath(), "cold_display_probe"});
    QVERIFY(cold.waitForFinished(90000));
    QCOMPARE(cold.exitCode(), 0);
    QVERIFY(cold.readAllStandardOutput().contains("OFFLINE_IPV4_IPV6_DENIED"));
    QFile report(f.temp.path() + "/cold.json");
    QVERIFY(report.open(QIODevice::ReadOnly));
    const auto r = QJsonDocument::fromJson(report.readAll()).object();
    QVERIFY(r.value("appPid").toString() !=
            QString::number(QCoreApplication::applicationPid()));
    QVERIFY(r.value("workerPid").toString() != QString::number(worker));
    QCOMPARE(r.value("state").toObject().value("rooms"), first.value("rooms"));
    QCOMPARE(r.value("state").toObject().value("invocations").toString(),
             QString("0"));
    QFile after(f.workspace + "/workspace.json");
    QVERIFY(after.open(QIODevice::ReadOnly));
    QCOMPARE(after.readAll(), saved);
    save("cold-process-proof.json", r);
  }
  void ux_008_stale_approval_refreshes_without_retry() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot());
    TraceRecorder trace(*f.shell, *f.panel,
                        f.temp.path() + "/stale-trace.jsonl");
    QVERIFY(trace.valid());
    QVERIFY(f.ask());
    QVERIFY(f.start());
    QVERIFY(f.ask());
    QFile file(f.workspace + "/workspace.json");
    QVERIFY(file.open(QIODevice::ReadOnly));
    auto snapshot = QJsonDocument::fromJson(file.readAll()).object();
    file.close();
    snapshot.remove("contentSha256");
    snapshot["revision"] = snapshot.value("revision").toInt() + 1;
    auto rooms = snapshot.value("rooms").toArray();
    auto room = rooms[0].toObject();
    room["pinned"] = true;
    rooms[0] = room;
    snapshot["rooms"] = rooms;
    snapshot["contentSha256"] = QString::fromLatin1(
        QCryptographicHash::hash(
            QJsonDocument(snapshot).toJson(QJsonDocument::Compact),
            QCryptographicHash::Sha256)
            .toHex());
    QVERIFY(file.open(QIODevice::WriteOnly | QIODevice::Truncate));
    file.write(QJsonDocument(snapshot).toJson(QJsonDocument::Compact));
    file.close();
    const bool staleFinished = f.start();
    QFile::remove(out + "/stale-first-trace.jsonl");
    QFile::copy(f.temp.path() + "/stale-trace.jsonl",
                out + "/stale-first-trace.jsonl");
    QVERIFY(staleFinished);
    QTRY_COMPARE(f.get<QListWidget>("workspacePinnedTools")->count(), 1);
    QCOMPARE(f.shell->bridge().rooms().size(), 1);
    QVERIFY(f.get<QLabel>("directStatus")->text().contains("changed"));
    QVERIFY(!f.get<QPushButton>("directStart")->isEnabled());
    QVERIFY(f.ask());
    const bool restarted = f.start();
    QFile::remove(out + "/stale-trace.jsonl");
    QFile::copy(f.temp.path() + "/stale-trace.jsonl",
                out + "/stale-trace.jsonl");
    if (!restarted)
      save("stale-timeout-state.json",
           {{"state", f.shell->bridge().lastState()},
            {"diagnostics",
             f.get<QPlainTextEdit>("operatorDiagnostics")->toPlainText()},
            {"busy", f.panel->busy()},
            {"bridgeBusy", f.shell->bridge().busy()},
            {"switching", f.shell->bridge().switching()},
            {"status", f.get<QLabel>("directStatus")->text()}});
    QVERIFY(restarted);
    QCOMPARE(f.shell->bridge().rooms().size(), 2);
  }
  void ux_009_optin_trace_is_private_passive_and_allowlisted() {
    Fixture f;
    QVERIFY(f.install());
    QVERIFY(f.boot(true));
    const auto path = f.temp.path() + "/trace.jsonl";
    TraceRecorder trace(*f.shell, *f.panel, path);
    QVERIFY(trace.valid());
    QVERIFY(f.ask());
    QVERIFY(f.start());
    QVERIFY(trace.valid());
    trace.nativeEvent(
        "untrusted-extra",
        {{"ok", false},
         {"code", "AUTH_FAILED"},
         {"rawProviderBody", QString::fromUtf8(fakeToken())},
         {"credentialSource", f.temp.path() + "/private-source"}});
    QFile data(path);
    QVERIFY(data.open(QIODevice::ReadOnly));
    const auto bytes = data.readAll();
    QVERIFY(!bytes.contains(fakeToken()));
    QVERIFY(!bytes.contains("private-source"));
    QVERIFY(!bytes.contains("I need a mortgage"));
    QVERIFY(bytes.contains("proposalDigest"));
    QVERIFY(bytes.contains("gesture.directStart"));
    QVERIFY(bytes.contains("1461.48"));
    struct stat st{};
    QVERIFY(::stat(path.toLocal8Bit().constData(), &st) == 0);
    QCOMPARE(st.st_mode & 0077, mode_t(0));
    TraceRecorder duplicate(*f.shell, *f.panel, path);
    QVERIFY(!duplicate.valid());
    const auto alias = f.temp.path() + "/alias";
    QVERIFY(::symlink(path.toLocal8Bit().constData(),
                      alias.toLocal8Bit().constData()) == 0);
    TraceRecorder link(*f.shell, *f.panel, alias);
    QVERIFY(!link.valid());
    const auto publicDir = f.temp.path() + "/public";
    QVERIFY(QDir().mkdir(publicDir));
    QVERIFY(::chmod(publicDir.toLocal8Bit().constData(), 0755) == 0);
    TraceRecorder publicLog(*f.shell, *f.panel, publicDir + "/trace");
    QVERIFY(!publicLog.valid());
    QFile::remove(out + "/passive-trace.jsonl");
    QVERIFY(QFile::copy(path, out + "/passive-trace.jsonl"));
  }
  void cold_display_probe() {
    if (qEnvironmentVariableIsEmpty("WB_UX_COLD_REGISTRY"))
      return;
    Fixture f;
    f.registry = qEnvironmentVariable("WB_UX_COLD_REGISTRY");
    f.workspace = qEnvironmentVariable("WB_UX_COLD_WORKSPACE");
    QVERIFY(f.open(false));
    QTRY_COMPARE_WITH_TIMEOUT(f.shell->bridge().committedValue("payment"),
                              QString("1753.77"), 60000);
    QCOMPARE(f.shell->bridge().lastState().value("invocations").toString(),
             QString("0"));
    QCOMPARE(f.panel->providerState().value("remoteAttempts").toString(),
             QString("0"));
    QCOMPARE(f.get<QListWidget>("workspacePinnedTools")->count(), 1);
    QFile report(qEnvironmentVariable("WB_UX_COLD_REPORT"));
    QVERIFY(report.open(QIODevice::WriteOnly));
    report.write(
        QJsonDocument(
            QJsonObject{
                {"appPid", QString::number(QCoreApplication::applicationPid())},
                {"workerPid", QString::number(f.shell->bridge().workerPid())},
                {"state", f.shell->bridge().lastState()},
                {"provider", f.panel->providerState()}})
            .toJson());
    report.close();
    QVERIFY(f.close());
  }
};
int main(int argc, char **argv) {
  QApplication app(argc, argv);
  QQuickStyle::setStyle("Basic");
  UxTests tests;
  auto args = app.arguments();
  bool selected = false;
  for (const auto &arg : args)
    if (arg.startsWith("ux_") || arg == "cold_display_probe")
      selected = true;
  if (!selected)
    for (int i = tests.metaObject()->methodOffset();
         i < tests.metaObject()->methodCount(); ++i) {
      const auto name =
          QString::fromLatin1(tests.metaObject()->method(i).name());
      if (name.startsWith("ux_") && !name.endsWith("_data"))
        args << name;
    }
  return QTest::qExec(&tests, args);
}
#include "ux_tests.moc"
