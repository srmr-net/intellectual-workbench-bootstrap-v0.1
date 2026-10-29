#include "WorkspaceView.h"
#include <QAccessible>
#include <QDateTime>
#include <QHBoxLayout>
#include <QJsonDocument>
#include <QLabel>
#include <QListWidget>
#include <QPlainTextEdit>
#include <QProcess>
#include <QPushButton>
#include <QSignalBlocker>
#include <QStackedWidget>
#include <QVBoxLayout>
namespace {
QLabel *heading(const QString &s, QWidget *parent) {
  auto *l = new QLabel(s, parent);
  l->setTextFormat(Qt::PlainText);
  l->setWordWrap(true);
  return l;
}
} // namespace
QString WorkspaceView::toolName(QString id) {
  id = id.section('.', -1);
  if (id.endsWith("-ui"))
    id.chop(3);
  id.replace('-', ' ');
  if (!id.isEmpty())
    id[0] = id[0].toUpper();
  return id;
}
WorkspaceView::WorkspaceView(DesktopShell &shell, QWidget *components,
                             QObject *parent)
    : QObject(parent), m_shell(shell) {
  shell.setWindowTitle("Intellectual Workbench");
  auto *left = shell.findChild<QWidget *>("leftPinnedTools");
  auto *nav = qobject_cast<QVBoxLayout *>(left->layout());
  m_installed = shell.findChild<QListWidget *>("installedComponents");
  auto *open = shell.findChild<QPushButton *>("openComponent");
  m_workspaces = shell.findChild<QListWidget *>("pinnedRooms");
  // Move the existing live registry view; no duplicate discovery/store/client.
  nav->removeWidget(m_installed);
  nav->removeWidget(open);
  auto *componentLayout = new QVBoxLayout(components);
  componentLayout->addWidget(
      heading("Installed tools and exact package versions", components));
  componentLayout->addWidget(m_installed);
  m_installed->setAccessibleName("Installed components and versions");
  // Component inspection never creates a room or bypasses Operator approval.
  QObject::disconnect(m_installed, &QListWidget::itemDoubleClicked, open,
                      &QPushButton::click);
  open->setParent(components);
  open->hide();
  m_componentInfo = new QPlainTextEdit(components);
  m_componentInfo->setObjectName("componentInformation");
  m_componentInfo->setAccessibleName(
      "Component identity, version and local registry information");
  m_componentInfo->setReadOnly(true);
  componentLayout->addWidget(m_componentInfo);
  if (auto *p = shell.bridge().findChild<QProcess *>())
    m_registry = p->arguments().value(0);
  connect(m_installed, &QListWidget::currentRowChanged, this,
          [this] { componentInformation(); });
  for (auto *l :
       left->findChildren<QLabel *>(QString(), Qt::FindDirectChildrenOnly)) {
    if (l->text() == "Installed components")
      l->setText("Workspaces");
    else if (l->text() == "Local rooms and pinned tools")
      l->hide();
  }
  m_workspaces->setAccessibleName("Workspaces");
  m_pinned = new QListWidget(left);
  m_pinned->setObjectName("workspacePinnedTools");
  m_pinned->setAccessibleName("Pinned tools");
  m_pinned->setMaximumHeight(150);
  m_recent = new QListWidget(left);
  m_recent->setObjectName("workspaceRecentActivity");
  m_recent->setAccessibleName("Recent activity this session");
  m_recent->setMaximumHeight(130);
  auto *pin = shell.findChild<QPushButton *>("pinCanvas");
  const int at = nav->indexOf(pin);
  nav->insertWidget(at, heading("Pinned tools", left));
  nav->insertWidget(at + 1, m_pinned);
  nav->insertWidget(at + 2, heading("Recent activity", left));
  nav->insertWidget(at + 3, m_recent);
  auto select = [this](QListWidgetItem *i) {
    const auto id = i->data(Qt::UserRole).toString();
    if (!id.isEmpty() && id != m_shell.bridge().selectedRoom())
      m_shell.bridge().selectRoom(id);
  };
  connect(m_pinned, &QListWidget::itemClicked, this, select);
  connect(m_pinned, &QListWidget::itemActivated, this, select);
  connect(m_recent, &QListWidget::itemActivated, this, select);
  auto *centre = shell.findChild<QWidget *>("activityContext");
  auto *context = qobject_cast<QVBoxLayout *>(centre->layout());
  for (auto *l :
       centre->findChildren<QLabel *>(QString(), Qt::FindDirectChildrenOnly)) {
    if (l->objectName() == "activityStatus" ||
        l->objectName() == "selectedIdentity" ||
        l->objectName() == "recoveryStatus") {
      context->removeWidget(l);
      componentLayout->addWidget(l);
      l->show();
    } else if (l->text() == "Activity and context")
      l->setText("Operator");
  }
  centre->setMinimumWidth(320);
  centre->setMaximumWidth(430);
  m_workspaceStatus =
      heading("Choose a workspace or ask Operator to start a tool.", left);
  m_workspaceStatus->setObjectName("workspaceStatus");
  nav->insertWidget(1, m_workspaceStatus);
  // Preserve the exact accepted semantic renderer/window and its bridge.
  auto *canvas = shell.findChild<QWidget *>("semanticCanvas");
  auto *columns = qobject_cast<QHBoxLayout *>(shell.layout());
  columns->removeWidget(canvas);
  auto *area = new QWidget(&shell);
  area->setObjectName("workspaceCanvas");
  auto *v = new QVBoxLayout(area);
  m_canvasTitle = heading("Canvas", area);
  m_canvasTitle->setObjectName("workspaceCanvasTitle");
  v->addWidget(m_canvasTitle);
  m_canvasStack = new QStackedWidget(area);
  m_canvasStack->setObjectName("workspaceCanvasStack");
  m_empty = heading("Your tools appear here. Ask Operator for a tool, then "
                    "choose Start this tool.",
                    m_canvasStack);
  m_empty->setObjectName("workspaceEmptyCanvas");
  m_empty->setAlignment(Qt::AlignCenter);
  m_empty->setMinimumWidth(430);
  m_canvasStack->addWidget(m_empty);
  m_canvasStack->addWidget(canvas);
  v->addWidget(m_canvasStack, 1);
  columns->addWidget(area, 1);
  connect(&shell.bridge(), &DesktopBridge::stateChanged, this,
          &WorkspaceView::refresh);
  connect(&shell.bridge(), &DesktopBridge::busyChanged, this,
          &WorkspaceView::refresh);
  connect(&shell.bridge(), &DesktopBridge::documentChanged, this,
          &WorkspaceView::refresh);
  connect(&shell.bridge(), &DesktopBridge::errorChanged, this,
          &WorkspaceView::refresh);
  refresh();
}
void WorkspaceView::componentInformation() {
  QString info = "Local immutable registry\n" + m_registry + "\n\n";
  if (auto *item = m_installed->currentItem())
    info +=
        QString::fromUtf8(QJsonDocument(QJsonObject::fromVariantMap(
                                            item->data(Qt::UserRole).toMap()))
                              .toJson());
  else
    info += "No installed tools are available.";
  info += "\nRegistry admission, exact pins and package permissions remain "
          "host-validated. This page is read-only.";
  if (m_componentInfo->toPlainText() != info)
    m_componentInfo->setPlainText(info);
}
void WorkspaceView::activity(const QString &text, const QString &room) {
  auto *item = new QListWidgetItem(
      QDateTime::currentDateTime().toString("HH:mm") + "  " + text.left(180));
  item->setData(Qt::UserRole, room);
  m_recent->insertItem(0, item);
  while (m_recent->count() > 20)
    delete m_recent->takeItem(m_recent->count() - 1);
}
void WorkspaceView::refresh() {
  const auto rooms = m_shell.bridge().rooms();
  const auto selected = m_shell.bridge().selectedRoom();
  const bool changed = rooms != m_lastRooms;
  if (changed) {
    QSignalBlocker block(m_pinned);
    const auto prior =
        m_pinned->currentItem()
            ? m_pinned->currentItem()->data(Qt::UserRole).toString()
            : QString();
    const auto restore = m_pinned->hasFocus() ? prior : selected;
    m_pinned->clear();
    for (int i = 0; i < rooms.size(); ++i) {
      const auto r = rooms[i].toMap();
      const auto name =
          toolName(r.value("component").toMap().value("id").toString());
      const auto title = "Workspace " + QString::number(i + 1) +
                         (name.isEmpty() ? QString() : " · " + name);
      if (i < m_workspaces->count())
        m_workspaces->item(i)->setText(title);
      if (r.value("pinned").toBool()) {
        auto *item =
            new QListWidgetItem(name.isEmpty() ? title : name, m_pinned);
        item->setData(Qt::UserRole, r.value("id"));
        if (r.value("id").toString() == restore)
          m_pinned->setCurrentItem(item);
      }
    }
    if (rooms.size() > m_lastRooms.size())
      activity("Opened a tool workspace", selected);
    m_lastRooms = rooms;
  }
  if (selected != m_selected) {
    QSignalBlocker block(m_pinned);
    for (int i = 0; i < m_pinned->count(); ++i)
      if (m_pinned->item(i)->data(Qt::UserRole).toString() == selected)
        m_pinned->setCurrentRow(i);
    m_selected = selected;
  }
  m_pinned->setEnabled(!m_shell.bridge().switching() &&
                       !m_shell.bridge().quitting());
  bool pinned = false;
  for (const auto &entry : rooms) {
    const auto r = entry.toMap();
    if (r.value("id") == m_shell.bridge().selectedRoom())
      pinned = r.value("pinned").toBool();
  }
  m_shell.findChild<QPushButton *>("pinCanvas")
      ->setText(pinned ? "Unpin tool" : "Pin tool");
  const bool hasTool = !m_shell.bridge().controls().isEmpty();
  m_canvasStack->setCurrentIndex(hasTool ? 1 : 0);
  m_canvasTitle->setText(hasTool ? m_shell.bridge().title() : "Canvas");
  const bool recovery =
      m_shell.bridge().lastState().value("recovery").isString();
  m_empty->setText(recovery ? "This workspace needs attention. Your saved "
                              "state is preserved. Open Settings for details."
                            : "Your tools appear here. Ask Operator for a "
                              "tool, then choose Start this tool.");
  const QString state =
      m_shell.bridge().lastState().isEmpty() ? "Opening your workspace…"
      : recovery ? "Workspace needs attention — see Settings."
      : m_shell.bridge().switching() ? "Opening workspace…"
      : m_shell.bridge().busy()
          ? "Updating tool… results show the last committed calculation."
      : m_shell.bridge().hasAnyDraft()
          ? "Unapplied edits. Review your draft; results show the last "
            "committed calculation."
      : !m_shell.bridge().error().isEmpty()
          ? "The last change needs attention. Check the form or Settings."
      : hasTool ? "Workspace open. Calculations run locally."
                : "Choose a workspace or ask Operator to start a tool.";
  if (m_workspaceStatus->text() != state) {
    m_workspaceStatus->setText(state);
    m_workspaceStatus->setAccessibleDescription(state);
    QAccessibleEvent event(m_workspaceStatus, QAccessible::Alert);
    QAccessible::updateAccessibility(&event);
  }
  componentInformation();
}
