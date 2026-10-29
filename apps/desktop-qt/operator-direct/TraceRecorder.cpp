#include "TraceRecorder.h"
#include "DirectPanel.h"
#include <QCoreApplication>
#include <QDateTime>
#include <QDir>
#include <QJsonArray>
#include <QJsonDocument>
#include <QPushButton>
#include <QTimer>
#include <fcntl.h>
#include <sys/stat.h>
#include <unistd.h>
namespace {
int privateNewFile(QString path) {
  if (!path.startsWith('/') || QDir::cleanPath(path) != path)
    return -1;
  auto parts = path.split('/', Qt::SkipEmptyParts);
  if (parts.isEmpty())
    return -1;
  const auto leaf = parts.takeLast().toLocal8Bit();
  int dir = ::open("/", O_RDONLY | O_DIRECTORY | O_CLOEXEC);
  for (const auto &part : parts) {
    if (dir < 0)
      return -1;
    const auto name = part.toLocal8Bit();
    const int next = ::openat(dir, name.constData(),
                              O_RDONLY | O_DIRECTORY | O_NOFOLLOW | O_CLOEXEC);
    ::close(dir);
    dir = next;
  }
  if (dir < 0)
    return -1;
  struct stat st{};
  if (::fstat(dir, &st) || st.st_uid != ::geteuid() || (st.st_mode & 0077)) {
    ::close(dir);
    return -1;
  }
  const int fd =
      ::openat(dir, leaf.constData(),
               O_WRONLY | O_CREAT | O_EXCL | O_NOFOLLOW | O_CLOEXEC, 0600);
  ::close(dir);
  return fd;
}
} // namespace
TraceRecorder::TraceRecorder(DesktopShell &shell, DirectPanel &panel,
                             QString file, QObject *parent)
    : QObject(parent), m_shell(shell), m_panel(panel) {
  const int fd = privateNewFile(file);
  if (fd < 0)
    return;
  if (!m_file.open(fd, QIODevice::WriteOnly, QFileDevice::AutoCloseHandle)) {
    ::close(fd);
    return;
  }
  connect(&panel, &DirectPanel::observed, this, &TraceRecorder::nativeEvent);
  connect(&shell.bridge(), &DesktopBridge::stateChanged, this,
          &TraceRecorder::state);
  connect(&shell.bridge(), &DesktopBridge::valuesChanged, this,
          &TraceRecorder::state);
  for (const auto &name : {"directAsk", "directStart", "directDismiss",
                           "directCancel", "pinCanvas"})
    if (auto *button = shell.findChild<QPushButton *>(name))
      connect(button, &QPushButton::clicked, this,
              [this, name] { write({{"event", QString("gesture.") + name}}); });
  connect(&shell.bridge(), &DesktopBridge::shutdownComplete, this,
          [this](bool accepted) {
            write({{"event", "trace.closed"}, {"accepted", accepted}});
          });
  write({{"event", "trace.started"}});
  state();
}
void TraceRecorder::write(QJsonObject value) {
  if (!valid())
    return;
  value["utc"] = QDateTime::currentDateTimeUtc().toString(Qt::ISODateWithMs);
  value["appPid"] = QString::number(QCoreApplication::applicationPid());
  value["workerPid"] = QString::number(m_shell.bridge().workerPid());
  const auto bytes = QJsonDocument(value).toJson(QJsonDocument::Compact) + '\n';
  if (bytes.size() > 1024 * 1024 || m_bytes + bytes.size() > 10 * 1024 * 1024 ||
      m_file.write(bytes) != bytes.size() || !m_file.flush()) {
    m_failed = true;
    return;
  }
  m_bytes += bytes.size();
}
void TraceRecorder::nativeEvent(const QString &phase,
                                const QJsonObject &reply) {
  const auto provider = reply.value("protocol") == "provider.1"
                            ? reply
                            : reply.value("providerState").toObject();
  const auto profile = provider.value("profile").toObject();
  QJsonObject record{{"event", "operator.reply"},
                     {"phase", phase},
                     {"ok", reply.value("ok")},
                     {"code", reply.value("code")},
                     {"provider", profile.value("provider")},
                     {"model", profile.value("model")},
                     {"remoteAttempts", provider.value("remoteAttempts")}};
  const auto audit =
      reply.value("providerAudit").toObject().value("observation").toObject();
  QJsonObject observation;
  for (const auto &key :
       {"observationVersion", "correlationId", "provider", "model", "status",
        "durationMs", "catalogCount", "inputTokens", "outputTokens"})
    if (audit.contains(key))
      observation[key] = audit.value(key);
  record["observation"] = observation;
  for (const auto &key : {"topologyCommitted", "topologyCheckpointPersisted",
                          "initializationSucceeded", "checkpointPersisted",
                          "roomId", "roomRevision", "initialInvocations"})
    if (reply.contains(key))
      record[key] = reply.value(key);
  if (reply.contains("preview")) {
    const auto preview = reply.value("preview").toObject(),
               plan = preview.value("plan").toObject();
    const auto instances = plan.value("instances").toArray();
    record["proposalDigest"] = preview.value("proposalDigest");
    record["initialInputs"] = preview.value("initialInputs");
    record["expectedRoomRevision"] = plan.value("expectedRoomRevision");
    record["roomId"] = plan.value("roomId");
    if (instances.size() == 1)
      record["component"] = instances[0].toObject().value("component");
  }
  // Never serialize raw prompts, explanations, source references, provider
  // bodies or whole reply/profile objects. Unknown future fields are excluded.
  write(record);
  QTimer::singleShot(0, this, &TraceRecorder::state);
}
void TraceRecorder::state() {
  const auto s = m_shell.bridge().lastState();
  if (s.isEmpty())
    return;
  const auto p = s.value("projection").toObject();
  QJsonArray rooms;
  for (const auto &v : s.value("rooms").toArray()) {
    const auto r = v.toObject();
    rooms.append(QJsonObject{{"id", r.value("id")},
                             {"component", r.value("component")},
                             {"pinned", r.value("pinned")}});
  }
  const auto provider = m_panel.providerState();
  write({{"event", "workspace.state"},
         {"selectedRoom", m_shell.bridge().selectedRoom()},
         {"rooms", rooms},
         {"values", p.value("values")},
         {"roomRevision", p.value("roomRevision")},
         {"invocations", s.value("invocations")},
         {"remoteAttempts", provider.value("remoteAttempts")}});
}
