#include "OperatorDriver.h"
#include <QAccessible>
#include <QAccessibleInterface>
#include <QApplication>
#include <QCheckBox>
#include <QComboBox>
#include <QCryptographicHash>
#include <QDir>
#include <QFile>
#include <QFileInfo>
#include <QInputMethodEvent>
#include <QJsonDocument>
#include <QLineEdit>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QPushButton>
#include <QQuickItem>
#include <QQuickWindow>
#include <QScrollArea>
#include <QTreeWidget>
#include <QtTest>
namespace {
int statusAlerts = 0;
QAccessible::UpdateHandler previousHandler = nullptr;
void accessibilityEvent(QAccessibleEvent *event) {
  if (event->type() == QAccessible::Alert && event->object() &&
      event->object()->objectName() == "operatorStatus")
    ++statusAlerts;
  if (previousHandler)
    previousHandler(event);
}

void reveal(QWidget *widget) {
  for (auto *parent = widget->parentWidget(); parent;
       parent = parent->parentWidget())
    if (auto *scroll = qobject_cast<QScrollArea *>(parent))
      scroll->ensureWidgetVisible(widget);
}

QQuickItem *find(QQuickItem *root, const QString &name) {
  if (root->objectName() == name)
    return root;
  for (auto *c : root->childItems())
    if (auto *v = find(c, name))
      return v;
  return nullptr;
}
} // namespace
OperatorDriver::OperatorDriver(DesktopShell &shell, OperatorPanel &panel,
                               QString actions, QString report)
    : m_shell(shell), m_operator(panel), m_actions(std::move(actions)),
      m_report(std::move(report)) {}
void OperatorDriver::start() {
  QAccessible::setActive(true);
  previousHandler = QAccessible::installUpdateHandler(accessibilityEvent);
  m_deadline.start();
  QTimer::singleShot(100, &m_shell, [this] { run(); });
}
bool OperatorDriver::wait(const std::function<bool()> &predicate, int ms) {
  QElapsedTimer timer;
  timer.start();
  while (!predicate()) {
    if (timer.elapsed() > ms || m_deadline.elapsed() > 300000)
      return false;
    QTest::qWait(20);
  }
  return true;
}
void OperatorDriver::sample(const QString &action) {
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
  QJsonArray operatorWidgets;
  for (const QString &name :
       {QString("credentialPath"), QString("credentialApply"),
        QString("operatorSend"), QString("outboundData"),
        QString("proposalDetails"), QString("providerChoice"),
        QString("remoteConsent"), QString("providerInfo"),
        QString("operatorIntent"), QString("operatorSubmitted"),
        QString("operatorStatus"), QString("operatorDecline"),
        QString("operatorPreview"), QString("operatorConsent"),
        QString("operatorApprove"), QString("operatorCancel")}) {
    auto *widget = m_shell.findChild<QWidget *>(name);
    auto *a = widget ? QAccessible::queryAccessibleInterface(widget) : nullptr;
    operatorWidgets.append(
        QJsonObject{{"id", name},
                    {"name", a ? a->text(QAccessible::Name) : QString()},
                    {"role", a ? int(a->role()) : -1},
                    {"enabled", widget && widget->isEnabled()},
                    {"visible", widget && widget->isVisible()},
                    {"focusable", a ? bool(a->state().focusable) : false}});
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
  m_samples.append(QJsonObject{
      {"action", action},
      {"disclosure", m_operator.disclosure()},
      {"uxReply", m_operator.uxReply()},
      {"statusAlerts", statusAlerts},
      {"outboundText",
       m_shell.findChild<QPlainTextEdit *>("outboundData")->toPlainText()},
      {"state", bridge.lastState()},
      {"operatorReply", m_operator.lastReply()},
      {"providerState", m_operator.providerState()},
      {"remoteConsent",
       m_shell.findChild<QCheckBox *>("remoteConsent")->isChecked()},
      {"operatorWidgets", operatorWidgets},
      {"operatorStatus", m_operator.error()},
      {"operatorBusy", m_operator.busy()},
      {"operatorGuiThread", m_operator.guiThread()},
      {"hasPreview", m_operator.hasPreview()},
      {"consentChecked",
       m_shell.findChild<QCheckBox *>("operatorConsent")->isChecked()},
      {"approveEnabled",
       m_shell.findChild<QPushButton *>("operatorApprove")->isEnabled()},
      {"submittedText",
       m_shell.findChild<QPlainTextEdit *>("operatorSubmitted")->toPlainText()},
      {"previewText",
       m_shell.findChild<QPlainTextEdit *>("operatorPreview")->toPlainText()},
      {"error", bridge.error()},
      {"controls", controls},
      {"columns", columns},
      {"guiThread", bridge.callbacksOnGuiThread()},
      {"windowActive", m_shell.canvasWindow()->isActive()},
      {"windowVisible", m_shell.canvasWindow()->isVisible()},
      {"ready", bridge.ready()},
      {"pending", bridge.pendingCount()}});
}
void OperatorDriver::run() {
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
  if (!wait([&] {
        return !m_operator.busy() && !m_operator.lastReply().isEmpty();
      })) {
    fail("operator startup timeout");
    return;
  }
  m_initialProviderCalls =
      m_operator.lastReply().value("providerCalls").toString();
  m_initialGuestInvocations =
      bridge.lastState().value("invocations").toString();
  for (const auto &entry : doc.array()) {
    const auto action = entry.toObject();
    const QString type = action.value("type").toString();
    if (type == "credentialSource") {
      auto *editor = m_shell.findChild<QLineEdit *>("credentialPath");
      if (!editor || !editor->isEnabled()) {
        fail("credential source unavailable");
        return;
      }
      reveal(editor);
      editor->setFocus();
      QTest::keyClicks(editor, action.value("path").toString());
      QTest::mouseClick(m_shell.findChild<QPushButton *>("credentialApply"),
                        Qt::LeftButton);
      if (!wait([&] { return !m_operator.busy(); }) ||
          !editor->text().isEmpty()) {
        fail("credential source update failed");
        return;
      }
    } else if (type == "sendReviewed") {
      auto *send = m_shell.findChild<QPushButton *>("operatorSend");
      if (!send->isEnabled()) {
        fail("reviewed send unavailable");
        return;
      }
      reveal(send);
      send->setFocus();
      QTest::keyClick(send, Qt::Key_Space);
      if (action.value("settle").toBool(true) &&
          !wait([&] { return !m_operator.busy(); })) {
        fail("remote response timeout");
        return;
      }
    } else if (type == "wait") {
      QTest::qWait(qBound(0, action.value("milliseconds").toInt(), 35000));
    } else if (type == "tabJourney") {
      QApplication::setActiveWindow(&m_shell);
      auto *start = m_shell.findChild<QComboBox *>("providerChoice");
      reveal(start);
      start->setFocus();
      QTest::qWait(30);
      QJsonArray names;
      bool reached = false;
      for (int i = 0; i < 40; ++i) {
        auto *focus = QApplication::focusWidget();
        if (!focus) {
          m_samples.append(QJsonObject{{"action", "focusTraversalFailed"},
                                       {"names", names}});
          fail("keyboard focus lost");
          return;
        }
        names.append(focus->objectName());
        if (focus->objectName() == "operatorConsent") {
          QTest::keyClick(focus, Qt::Key_Space);
          QTest::keyClick(focus, Qt::Key_Tab);
          auto *next = QApplication::focusWidget();
          if (!next || next->objectName() != "operatorApprove") {
            fail("approval tab order");
            return;
          }
          names.append(next->objectName());
          focus->setFocus();
          QTest::keyClick(focus, Qt::Key_Space);
          reached = true;
          break;
        }
        reveal(focus);
        QTest::keyClick(focus, Qt::Key_Tab);
        QTest::qWait(10);
      }
      if (!reached) {
        fail("confirmation unreachable by Tab");
        return;
      }
      m_samples.append(
          QJsonObject{{"action", "focusTraversal"}, {"names", names}});
    } else if (type == "selectProvider") {
      auto *choice = m_shell.findChild<QComboBox *>("providerChoice");
      reveal(choice);
      choice->setFocus();
      QTest::keyClick(choice, action.value("remote").toBool() ? Qt::Key_End
                                                              : Qt::Key_Home);
      if (!wait([&] { return !m_operator.busy(); })) {
        fail("provider selection timeout");
        return;
      }
    } else if (type == "awaitHumanInferenceConsent") {
      const auto review = m_operator.disclosure();
      const auto digest = review.value("reviewDigest").toString();
      const auto decisionPath = m_report + ".consent.json";
      if (!m_operator.hasDisclosure() || digest.isEmpty() ||
          QFile::exists(decisionPath)) {
        fail("fresh inference decision required");
        return;
      }
      QFile out(m_report + ".disclosure.json");
      if (!out.open(QIODevice::WriteOnly)) {
        fail("disclosure publication failed");
        return;
      }
      out.write(
          QJsonDocument(
              QJsonObject{
                  {"disclosure", review},
                  {"providerState", m_operator.providerState()},
                  {"pid", QString::number(QCoreApplication::applicationPid())}})
              .toJson());
      out.close();
      if (!wait([&] { return QFile::exists(decisionPath); }, 100000)) {
        fail("human inference consent expired");
        return;
      }
      QFile decision(decisionPath);
      if (!decision.open(QIODevice::ReadOnly) || decision.size() > 1024) {
        fail("invalid inference decision");
        return;
      }
      const auto object = QJsonDocument::fromJson(decision.readAll()).object();
      if (object.size() != 2 ||
          object.value("reviewDigest").toString() != digest ||
          object.value("decision").toString() != "consent") {
        fail("human denied or mismatched inference request");
        return;
      }
    } else if (type == "awaitHumanApproval") {
      // Explicit live-validation instrumentation, never normal application
      // flow. Publish the actual host preview, then wait for a separate human
      // decision.
      const auto preview = m_operator.lastReply().value("preview").toObject();
      const auto digest = preview.value("proposalDigest").toString();
      const auto decisionPath = m_report + ".approval.json";
      if (!m_operator.hasPreview() || digest.isEmpty() ||
          QFile::exists(decisionPath)) {
        fail("fresh human decision required");
        return;
      }
      QFile out(m_report + ".preview.json");
      if (!out.open(QIODevice::WriteOnly)) {
        fail("preview publication failed");
        return;
      }
      out.write(
          QJsonDocument(
              QJsonObject{
                  {"preview", preview},
                  {"providerState", m_operator.providerState()},
                  {"pid", QString::number(QCoreApplication::applicationPid())}})
              .toJson());
      out.close();
      if (!wait([&] { return QFile::exists(decisionPath); }, 100000)) {
        fail("human approval expired");
        return;
      }
      QFile decision(decisionPath);
      if (!decision.open(QIODevice::ReadOnly) || decision.size() > 1024) {
        fail("invalid human decision");
        return;
      }
      const auto object = QJsonDocument::fromJson(decision.readAll()).object();
      if (object.size() != 2 ||
          object.value("proposalDigest").toString() != digest ||
          object.value("decision").toString() != "approve") {
        fail("human rejected or mismatched proposal");
        return;
      }
    } else if (type == "remoteConsent") {
      auto *consent = m_shell.findChild<QCheckBox *>("remoteConsent");
      if (!consent->isEnabled() ||
          consent->isChecked() == action.value("consent").toBool()) {
        fail("remote consent state");
        return;
      }
      reveal(consent);
      consent->setFocus();
      QTest::keyClick(consent, Qt::Key_Space);
      if (!wait([&] { return !m_operator.busy(); })) {
        fail("provider consent timeout");
        return;
      }
    } else if (type == "previewDraft") {
      auto *prepare = m_shell.findChild<QPushButton *>("operatorPrepare");
      if (!prepare->isEnabled()) {
        fail("preview unavailable");
        return;
      }
      reveal(prepare);
      prepare->setFocus();
      QTest::keyClick(prepare, Qt::Key_Space);
      if (action.value("settle").toBool(true) &&
          !wait([&] { return !m_operator.busy(); })) {
        fail("preview timeout");
        return;
      }
    } else if (type == "operatorStatus") {
      m_operator.inspect();
      if (!wait([&] { return !m_operator.busy(); })) {
        fail("operator status timeout");
        return;
      }
    } else if (type == "intent" || type == "intentKeyboard" ||
               type == "changeIntent" || type == "draftIntent") {
      auto *editor = m_shell.findChild<QPlainTextEdit *>("operatorIntent");
      if (!editor->isEnabled() ||
          !m_shell.findChild<QPushButton *>("operatorPrepare")->isEnabled()) {
        fail("intent unavailable");
        return;
      }
      reveal(editor);
      editor->setFocus();
      QTest::keyClick(editor, Qt::Key_A, Qt::ControlModifier);
      QInputMethodEvent input;
      input.setCommitString(action.value("text").toString());
      QApplication::sendEvent(editor, &input);
      if (editor->toPlainText() != action.value("text").toString()) {
        fail("intent input mismatch");
        return;
      }
      if (type == "changeIntent" || type == "draftIntent") {
        if (!wait([&] { return !m_operator.busy(); })) {
          fail("intent consent revocation timeout");
          return;
        }
        if (m_operator.hasPreview() ||
            m_shell.findChild<QPushButton *>("operatorApprove")->isEnabled()) {
          fail("stale consent after intent edit");
          return;
        }
        sample(type);
        continue;
      }
      auto *prepare = m_shell.findChild<QPushButton *>("operatorPrepare");
      reveal(prepare);
      prepare->setFocus();
      if (type == "intentKeyboard")
        QTest::keyClick(prepare, Qt::Key_Space);
      else
        QTest::mouseClick(prepare, Qt::LeftButton);
      if (action.value("settle").toBool(true) &&
          !wait([&] { return !m_operator.busy(); })) {
        fail("intent timeout");
        return;
      }
    } else if (type == "approveWithoutConsent") {
      auto *approve = m_shell.findChild<QPushButton *>("operatorApprove");
      if (approve->isEnabled() ||
          m_shell.findChild<QCheckBox *>("operatorConsent")->isChecked()) {
        fail("implicit consent");
        return;
      }
      QTest::mouseClick(approve, Qt::LeftButton);
      QTest::qWait(100);
      if (!bridge.rooms().isEmpty()) {
        fail("unapproved mutation");
        return;
      }
    } else if (type == "approve" || type == "approveAndCancel" ||
               type == "approveKeyboard" || type == "doubleApprove") {
      auto *consent = m_shell.findChild<QCheckBox *>("operatorConsent");
      if (!m_operator.hasPreview() || consent->isChecked()) {
        fail("fresh consent unavailable");
        return;
      }
      reveal(consent);
      consent->setFocus();
      if (type == "approveKeyboard")
        QTest::keyClick(consent, Qt::Key_Space);
      else
        QTest::mouseClick(consent, Qt::LeftButton);
      auto *approve = m_shell.findChild<QPushButton *>("operatorApprove");
      if (!consent->isChecked() || !approve->isEnabled()) {
        fail("approval disabled");
        return;
      }
      reveal(approve);
      approve->setFocus();
      if (type == "approveKeyboard")
        QTest::keyClick(approve, Qt::Key_Space);
      else
        QTest::mouseClick(approve, Qt::LeftButton);
      if (type == "doubleApprove")
        QTest::mouseDClick(approve, Qt::LeftButton);
      if (type == "approveAndCancel") {
        auto *cancel = m_shell.findChild<QPushButton *>("operatorCancel");
        if (cancel->isEnabled()) {
          fail("cancel enabled after confirmation");
          return;
        }
        QTest::mouseClick(cancel, Qt::LeftButton);
      }
      if (!wait([&] {
            return !m_operator.busy() && !bridge.busy() && !bridge.switching();
          })) {
        fail("approval timeout");
        return;
      }
    } else if (type == "decline" || type == "cancel") {
      auto *control = m_shell.findChild<QPushButton *>(
          type == "decline" ? "operatorDecline" : "operatorCancel");
      if (!control->isEnabled()) {
        fail("request control disabled");
        return;
      }
      QTest::mouseClick(control, Qt::LeftButton);
      if (!wait([&] { return !m_operator.busy(); })) {
        fail("request control timeout");
        return;
      }
    } else if (type == "open") {
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
    } else if (type == "applyThenSelect") {
      auto *apply = find(m_shell.canvasWindow()->contentItem(), "applyDraft");
      auto *list = m_shell.findChild<QListWidget *>("pinnedRooms");
      const int index = action.value("index").toInt(-1);
      if (!apply || index < 0 || index >= list->count()) {
        fail("compound action unavailable");
        return;
      }
      const auto point =
          apply->mapToScene(QPointF(apply->width() / 2, apply->height() / 2))
              .toPoint();
      QTest::mouseClick(m_shell.canvasWindow(), Qt::LeftButton, Qt::NoModifier,
                        point);
      if (bridge.pendingCount() == 0) {
        fail("no in-flight ingress");
        return;
      }
      sample("inFlightApply");
      QTest::mouseClick(list->viewport(), Qt::LeftButton, Qt::NoModifier,
                        list->visualItemRect(list->item(index)).center());
      if (!wait([&] { return !bridge.switching() && !bridge.busy(); })) {
        fail("compound select timeout");
        return;
      }
    } else if (type == "applyIfConflict" &&
               !bridge.error().contains("STATE CONFLICT")) {
      // A serial completion needs no retry. A fenced draft requires the same
      // explicit trusted Apply control as a human recovery gesture.
    } else if (type == "apply" || type == "applyIfConflict") {
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
void OperatorDriver::finish(int exitCode) {
  sample("exit");
  QFile binary(QCoreApplication::applicationFilePath());
  if (!binary.open(QIODevice::ReadOnly))
    m_failure = "binary evidence unavailable";
  QJsonObject report{
      {"pid", QCoreApplication::applicationPid()},
      {"initialProviderCalls", m_initialProviderCalls},
      {"initialGuestInvocations", m_initialGuestInvocations},
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
