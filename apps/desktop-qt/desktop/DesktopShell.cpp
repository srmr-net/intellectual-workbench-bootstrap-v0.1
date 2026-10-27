#include "DesktopShell.h"
#include <QApplication>
#include <QCloseEvent>
#include <QHBoxLayout>
#include <QJsonArray>
#include <QJsonDocument>
#include <QKeyEvent>
#include <QLabel>
#include <QListWidget>
#include <QPushButton>
#include <QQmlContext>
#include <QQmlProperty>
#include <QQuickItem>
#include <QQuickWindow>
#include <QSignalBlocker>
#include <QVBoxLayout>
namespace {
QLabel *label(const QString &text, QWidget *parent) {
  auto *v = new QLabel(text, parent);
  v->setTextFormat(Qt::PlainText);
  v->setWordWrap(true);
  return v;
}
QQuickItem *findItem(QQuickItem *root, const QString &name) {
  if (root->objectName() == name)
    return root;
  for (auto *child : root->childItems())
    if (auto *found = findItem(child, name))
      return found;
  return nullptr;
}
} // namespace
DesktopShell::DesktopShell(const DesktopConfig &config, int capacity,
                           QWidget *parent)
    : QWidget(parent), m_bridge(this, capacity) {
  setObjectName("workbenchDesktop");
  setWindowTitle("Workbench — local canvas");
  resize(1280, 780);
  auto *columns = new QHBoxLayout(this);
  auto *left = new QWidget(this);
  left->setObjectName("leftPinnedTools");
  left->setMinimumWidth(230);
  left->setMaximumWidth(300);
  auto *leftLayout = new QVBoxLayout(left);
  leftLayout->addWidget(label("Installed components", left));
  m_installed = new QListWidget(left);
  m_installed->setObjectName("installedComponents");
  m_installed->setAccessibleName("Installed components");
  leftLayout->addWidget(m_installed);
  m_open = new QPushButton("Open new canvas", left);
  m_open->setObjectName("openComponent");
  leftLayout->addWidget(m_open);
  leftLayout->addWidget(label("Local rooms and pinned tools", left));
  m_rooms = new QListWidget(left);
  m_rooms->setObjectName("pinnedRooms");
  m_rooms->setAccessibleName("Local rooms and pinned tools");
  leftLayout->addWidget(m_rooms);
  m_pin = new QPushButton("Pin canvas", left);
  m_pin->setObjectName("pinCanvas");
  leftLayout->addWidget(m_pin);
  m_quit = new QPushButton("Quit Workbench", left);
  m_quit->setObjectName("quitWorkbench");
  leftLayout->addWidget(m_quit);
  columns->addWidget(left);
  auto *centre = new QWidget(this);
  centre->setObjectName("activityContext");
  centre->setMinimumWidth(230);
  centre->setMaximumWidth(320);
  auto *context = new QVBoxLayout(centre);
  context->addWidget(label("Activity and context", centre));
  m_status = label("Opening local workspace…", centre);
  m_status->setObjectName("activityStatus");
  m_status->setAccessibleName("Workspace status");
  context->addWidget(m_status);
  m_identity =
      label("Choose an installed component to create a canvas.", centre);
  m_identity->setObjectName("selectedIdentity");
  context->addWidget(m_identity);
  m_error = label("", centre);
  m_error->setObjectName("recoveryStatus");
  m_error->setAccessibleName("Workspace diagnostics");
  context->addWidget(m_error);
  context->addStretch();
  columns->addWidget(centre);
  m_engine.rootContext()->setContextProperty("bridge", &m_bridge);
  // Byte-preserved accepted generic renderer, embedded as a genuine Qt window.
  m_engine.load(QUrl("qrc:/qml/Main.qml"));
  if (!m_engine.rootObjects().isEmpty())
    m_canvas = qobject_cast<QQuickWindow *>(m_engine.rootObjects().first());
  if (!m_canvas)
    qFatal("TRUSTED_RENDERER_UNAVAILABLE");
  m_canvas->installEventFilter(this);
  m_container = QWidget::createWindowContainer(m_canvas, this);
  m_container->setObjectName("semanticCanvas");
  m_container->setMinimumWidth(500);
  m_container->setFocusPolicy(Qt::StrongFocus);
  columns->addWidget(m_container, 1);
  connect(m_open, &QPushButton::clicked, this, [this] {
    if (auto *i = m_installed->currentItem())
      m_bridge.createRoom(
          QJsonObject::fromVariantMap(i->data(Qt::UserRole).toMap()));
  });
  connect(m_installed, &QListWidget::itemDoubleClicked, m_open,
          &QPushButton::click);
  connect(m_rooms, &QListWidget::itemActivated, this,
          [this](QListWidgetItem *i) {
            m_bridge.selectRoom(i->data(Qt::UserRole).toString());
          });
  connect(m_rooms, &QListWidget::itemClicked, this, [this](QListWidgetItem *i) {
    if (i->data(Qt::UserRole).toString() != m_bridge.selectedRoom())
      m_bridge.selectRoom(i->data(Qt::UserRole).toString());
  });
  connect(m_pin, &QPushButton::clicked, this, [this] {
    for (const auto &r : m_bridge.rooms()) {
      const auto room = r.toMap();
      if (room.value("id") == m_bridge.selectedRoom()) {
        m_bridge.setPinned(!room.value("pinned").toBool());
        break;
      }
    }
  });
  connect(m_quit, &QPushButton::clicked, this, &QWidget::close);
  connect(&m_bridge, &DesktopBridge::stateChanged, this,
          &DesktopShell::refresh);
  connect(&m_bridge, &DesktopBridge::busyChanged, this, &DesktopShell::refresh);
  connect(&m_bridge, &DesktopBridge::valuesChanged, this, [this] {
    QTimer::singleShot(0, this, &DesktopShell::decorateFields);
  });
  connect(&m_bridge, &DesktopBridge::documentChanged, this, [this] {
    QTimer::singleShot(0, this, &DesktopShell::decorateFields);
  });
  connect(&m_bridge, &DesktopBridge::shutdownComplete, this, [this](bool ok) {
    m_shutdown = true;
    close();
    emit closed(ok);
  });
  m_bridge.start(config);
  refresh();
}
void DesktopShell::refresh() {
  const auto installed = m_bridge.installed();
  if (installed != m_previousInstalled) {
    QSignalBlocker block(m_installed);
    m_installed->clear();
    for (const auto &entry : installed) {
      const auto v = entry.toMap();
      auto *item = new QListWidgetItem(
          v.value("label").toString() + " @ " +
              v.value("pin").toMap().value("version").toString(),
          m_installed);
      item->setData(Qt::UserRole, v.value("pin"));
    }
    m_previousInstalled = installed;
    if (m_installed->count())
      m_installed->setCurrentRow(0);
  }
  const auto rooms = m_bridge.rooms();
  if (rooms != m_previousRooms) {
    QSignalBlocker block(m_rooms);
    m_rooms->clear();
    for (const auto &entry : rooms) {
      const auto r = entry.toMap();
      const QString prefix = r.value("pinned").toBool() ? "Pinned: " : "Room: ";
      auto *item =
          new QListWidgetItem(prefix + r.value("label").toString() + " (" +
                                  r.value("id").toString().right(6) + ")",
                              m_rooms);
      item->setData(Qt::UserRole, r.value("id"));
    }
    m_previousRooms = rooms;
  }
  for (int i = 0; i < m_rooms->count(); ++i)
    if (m_rooms->item(i)->data(Qt::UserRole).toString() ==
        m_bridge.selectedRoom())
      m_rooms->setCurrentRow(i);
  m_container->setEnabled(m_bridge.ready());
  m_open->setEnabled(!m_bridge.quitting() && !m_bridge.switching() &&
                     !m_bridge.lastState().isEmpty() &&
                     m_bridge.lastState().value("recovery").isNull());
  m_pin->setEnabled(m_bridge.ready());
  m_rooms->setEnabled(!m_bridge.quitting() && !m_bridge.switching());
  bool pinned = false;
  for (const auto &entry : rooms)
    if (entry.toMap().value("id") == m_bridge.selectedRoom())
      pinned = entry.toMap().value("pinned").toBool();
  m_pin->setText(pinned ? "Unpin canvas" : "Pin canvas");
  if (m_bridge.quitting())
    m_status->setText("Finishing accepted work and closing safely…");
  else if (m_bridge.switching())
    m_status->setText("Opening exact pinned canvas; prior view detached…");
  else if (m_bridge.busy())
    m_status->setText("Working. Showing the last committed result.");
  else if (m_bridge.lastState().value("recovery").isString())
    m_status->setText(
        "Recovery required. The original workspace is preserved.");
  else if (m_bridge.hasAnyDraft())
    m_status->setText("Unapplied draft. Showing the last committed result. "
                      "Review and choose Apply draft.");
  else if (m_bridge.ready())
    m_status->setText("Committed local canvas. No model or network is needed.");
  else
    m_status->setText("Choose an installed local component.");
  const auto diagnostic =
      m_bridge.error() == "QUEUE_FULL"
          ? QString("The edit queue is full. Wait, review your retained draft, "
                    "then choose Apply draft. (QUEUE_FULL)")
          : m_bridge.error();
  m_error->setText(diagnostic);
  m_error->setAccessibleDescription(diagnostic);
  for (const auto &entry : rooms) {
    const auto room = entry.toMap();
    if (room.value("id") == m_bridge.selectedRoom()) {
      const auto pin = room.value("component").toMap();
      m_identity->setText("Exact component: " + pin.value("id").toString() +
                          "\nVersion: " + pin.value("version").toString() +
                          "\nSHA-256: " + pin.value("sha256").toString() +
                          "\nRoom: " + m_bridge.selectedRoom());
      break;
    }
  }
}
void DesktopShell::decorateFields() {
  for (const auto &entry : m_bridge.controls()) {
    const auto c = entry.toMap();
    if (c.value("type") == "readout")
      continue;
    auto *item =
        findItem(m_canvas->contentItem(), "input_" + c.value("id").toString());
    if (!item)
      continue;
    const QString error = m_bridge.controlError(c.value("id").toString());
    // Fixed trusted attached property, literal data only; no binding
    // expressions.
    QQmlProperty description(item, "Accessible.description",
                             QQmlEngine::contextForObject(item));
    if (!description.isValid() || !description.write(error))
      qFatal("ACCESSIBLE_DESCRIPTION_UNAVAILABLE");
    const auto normal =
        QQmlProperty(item, "palette.text", &m_engine).read().value<QColor>();
    item->setProperty("color",
                      error.isEmpty()
                          ? (normal.isValid() ? normal : QColor("#20242b"))
                          : QColor("#a00000"));
  }
}
void DesktopShell::closeEvent(QCloseEvent *event) {
  if (m_shutdown) {
    event->accept();
    return;
  }
  event->ignore();
  m_bridge.quit();
}

bool DesktopShell::eventFilter(QObject *object, QEvent *event) {
  // Native QWidget/QQuickWindow embedding can otherwise transfer Tab to the
  // widget chain before the accepted QML form sees it. Follow admitted order.
  if (object == m_canvas && event->type() == QEvent::KeyPress &&
      m_bridge.ready()) {
    auto *key = static_cast<QKeyEvent *>(event);
    if (key->key() == Qt::Key_Tab || key->key() == Qt::Key_Backtab) {
      QStringList names;
      for (const auto &v : m_bridge.controls())
        if (v.toMap().value("type") != "readout")
          names.append("input_" + v.toMap().value("id").toString());
      names.append("applyDraft");
      const auto *focus = m_canvas->activeFocusItem();
      const int index = focus ? names.indexOf(focus->objectName()) : -1;
      const bool reverse = key->key() == Qt::Key_Backtab ||
                           key->modifiers().testFlag(Qt::ShiftModifier);
      const int next = index + (reverse ? -1 : 1);
      if (index >= 0 && next >= 0 && next < names.size()) {
        if (auto *item = findItem(m_canvas->contentItem(), names[next])) {
          item->forceActiveFocus(reverse ? Qt::BacktabFocusReason
                                         : Qt::TabFocusReason);
          return true;
        }
      }
    }
  }
  return QWidget::eventFilter(object, event);
}
