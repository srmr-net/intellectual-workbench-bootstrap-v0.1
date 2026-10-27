#pragma once
#include "DesktopShell.h"
#include <QJsonArray>
#include <QJsonObject>
// Explicit native test instrumentation. Whitelisted UI actions only; no
// scripts, property expressions, guest commands or normal-mode background test
// behavior.
class ConformanceDriver {
public:
  ConformanceDriver(DesktopShell &, QString actions, QString report);
  void start();
  void finish(int exitCode);

private:
  bool wait(const std::function<bool()> &predicate, int milliseconds = 60000);
  void run();
  void sample(const QString &action);
  DesktopShell &m_shell;
  QString m_actions, m_report, m_failure;
  QJsonArray m_samples;
  QElapsedTimer m_deadline;
};
