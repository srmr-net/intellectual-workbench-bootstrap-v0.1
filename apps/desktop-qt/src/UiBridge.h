#pragma once
#include <QElapsedTimer>
#include <QJsonObject>
#include <QMap>
#include <QObject>
#include <QPointer>
#include <QProcess>
#include <QTimer>
#include <QVariantList>
#include <QVariantMap>

struct LaunchConfig {
  QString worker, source, store, python, semantic, abi, room;
  bool restore = false;
};
class UiBridge final : public QObject {
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
  explicit UiBridge(QObject *parent = nullptr, int capacity = 128);
  ~UiBridge() override;
  void start(const LaunchConfig &config);
  void stop();
  QVariantList controls() const { return m_controls; }
  QString title() const { return m_title; }
  QString error() const { return m_error; }
  bool ready() const { return m_ready; }
  QString viewEpoch() const { return QString::number(m_epoch); }
  bool darkMode() const { return m_dark; }
  qreal textScale() const { return m_scale; }
  void setDarkMode(bool dark);
  void setTextScale(qreal scale);
  Q_INVOKABLE QString committedValue(const QString &id) const;
  Q_INVOKABLE bool hasDraft(const QString &id) const;
  Q_INVOKABLE void setLocalDraft(const QString &id, const QString &text,
                                 const QString &epoch);
  Q_INVOKABLE void edit(const QString &id, const QString &text,
                        const QString &epoch);
  Q_INVOKABLE void apply(const QString &epoch);
  Q_INVOKABLE void save();
  QJsonObject lastProjection() const { return m_lastProjection; }
  quint64 appliedCount() const { return m_applied; }
  quint64 rejectedCount() const { return m_rejected; }
  int pendingCount() const { return m_pending.size(); }
  bool callbacksOnGuiThread() const { return m_threadOk; }
signals:
  void documentChanged();
  void valuesChanged();
  void errorChanged();
  void readyChanged();
  void themeChanged();
  void projectionApplied();
  void saved();

private:
  friend class UiConformance;
  struct Pending {
    QMap<QString, quint64> versions;
    qint64 since = 0;
  };
  void assertGui() const;
  void fail(const QString &code);
  void read(QProcess *process, quint64 epoch);
  void accept(const QJsonObject &frame, quint64 epoch);
  void send(const QJsonObject &command);
  static bool counter(const QJsonValue &value, quint64 &number);
  QProcess *m_process = nullptr;
  QByteArray m_buffer;
  QVariantList m_controls;
  QString m_title, m_error, m_expectedRoom;
  QJsonObject m_key, m_lastProjection;
  QVariantMap m_values;
  QMap<QString, QString> m_drafts;
  QMap<QString, quint64> m_draftVersions;
  QMap<QString, Pending> m_pending;
  quint64 m_epoch = 0, m_request = 0, m_revision = 0, m_generation = 0,
          m_sequence = 0, m_applied = 0, m_rejected = 0, m_draftClock = 0;
  qint64 m_started = 0;
  int m_capacity;
  bool m_ready = false, m_dark = false;
  mutable bool m_threadOk = true;
  qreal m_scale = 1.0;
  QElapsedTimer m_clock;
  QTimer m_watchdog;
};
