#pragma once
#include <QElapsedTimer>
#include <QJsonObject>
#include <QMap>
#include <QObject>
#include <QPointer>
#include <QProcess>
#include <QTimer>
#include <QVariantList>
struct DesktopConfig {
  QString worker, registry, workspace, python, semantic, abi;
};
class DesktopBridge final : public QObject {
  Q_OBJECT
  Q_PROPERTY(QVariantList controls READ controls NOTIFY documentChanged)
  Q_PROPERTY(QString title READ title NOTIFY documentChanged)
  Q_PROPERTY(QString error READ error NOTIFY errorChanged)
  Q_PROPERTY(bool ready READ ready NOTIFY readyChanged)
  Q_PROPERTY(QString viewEpoch READ viewEpoch NOTIFY documentChanged)
  Q_PROPERTY(bool darkMode READ darkMode WRITE setDarkMode NOTIFY themeChanged)
  Q_PROPERTY(
      qreal textScale READ textScale WRITE setTextScale NOTIFY themeChanged)
public:
  explicit DesktopBridge(QObject *parent = nullptr, int capacity = 32);
  ~DesktopBridge() override;
  void start(const DesktopConfig &config);
  QVariantList controls() const { return m_controls; }
  QString title() const { return m_title; }
  QString error() const { return m_error; }
  bool ready() const { return m_ready && !m_switching && !m_quitting; }
  QString viewEpoch() const { return QString::number(m_viewEpoch); }
  bool darkMode() const { return m_dark; }
  qreal textScale() const { return m_scale; }
  void setDarkMode(bool);
  void setTextScale(qreal);
  Q_INVOKABLE QString committedValue(const QString &) const;
  Q_INVOKABLE bool hasDraft(const QString &) const;
  Q_INVOKABLE void setLocalDraft(const QString &, const QString &,
                                 const QString &);
  Q_INVOKABLE void edit(const QString &, const QString &, const QString &);
  Q_INVOKABLE void apply(const QString &);
  Q_INVOKABLE void save();
  void createRoom(const QJsonObject &pin);
  void selectRoom(const QString &id);
  void setPinned(bool);
  void quit();
  QVariantList installed() const { return m_installed; }
  QVariantList rooms() const { return m_rooms; }
  QString selectedRoom() const { return m_selected; }
  bool switching() const { return m_switching; }
  bool busy() const { return !m_pending.isEmpty(); }
  bool hasAnyDraft() const { return !m_drafts.isEmpty(); }
  bool quitting() const { return m_quitting; }
  qint64 workerPid() const { return m_workerPid; }
  int pendingCount() const { return m_pending.size(); }
  QString controlError(const QString &id) const {
    return m_fieldErrors.value(id);
  }
  QJsonObject lastState() const { return m_lastState; }
  bool callbacksOnGuiThread() const { return m_threadOk; }
  quint64 rejectedCount() const { return m_rejected; }
signals:
  void documentChanged();
  void valuesChanged();
  void errorChanged();
  void readyChanged();
  void themeChanged();
  void stateChanged();
  void busyChanged();
  void commandFinished(QString, bool);
  void shutdownComplete(bool);

private:
  friend class Vs2Conformance;
  struct Pending {
    quint64 view;
    QMap<QString, quint64> versions;
    qint64 since;
    bool select = false;
  };
  void assertGui() const;
  void fail(const QString &);
  QString send(const QJsonObject &, bool selection = false,
               QMap<QString, quint64> versions = {});
  void sendUi(const QJsonObject &, QMap<QString, quint64> versions);
  void read();
  void accept(const QJsonObject &);
  static bool counter(const QJsonValue &, quint64 &);
  void selection(const QJsonObject &);
  void continueQuit();
  QProcess *m_process = nullptr;
  QTimer m_watchdog;
  QElapsedTimer m_clock;
  qint64 m_started = 0, m_workerPid = 0, m_quitStarted = 0;
  QByteArray m_buffer;
  QMap<QString, Pending> m_pending;
  QMap<QString, QString> m_drafts, m_fieldErrors;
  QMap<QString, quint64> m_versions;
  QVariantList m_controls, m_installed, m_rooms;
  QVariantMap m_values;
  QString m_title, m_error, m_selected, m_quitToken;
  QJsonObject m_key, m_lastState;
  quint64 m_request = 0, m_hostEpoch = 1, m_viewEpoch = 1, m_revision = 0,
          m_generation = 1, m_sequence = 0, m_draftClock = 0, m_rejected = 0;
  int m_capacity;
  bool m_ready = false, m_switching = false, m_quitting = false,
       m_quitAck = false, m_dark = false;
  qreal m_scale = 1;
  mutable bool m_threadOk = true;
};
