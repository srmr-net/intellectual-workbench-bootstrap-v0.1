QT += core gui widgets qml quick quickcontrols2
CONFIG += c++17 warn_on
TARGET = workbench-operator-direct
SOURCES += main.cpp DirectPanel.cpp WorkspaceView.cpp TraceRecorder.cpp ../desktop/DesktopShell.cpp ../desktop/DesktopBridge.cpp
HEADERS += DirectPanel.h WorkspaceView.h TraceRecorder.h ../desktop/DesktopShell.h ../desktop/DesktopBridge.h
RESOURCES += resources.qrc
