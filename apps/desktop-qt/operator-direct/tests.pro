QT += core gui widgets qml quick quickcontrols2 testlib
CONFIG += c++17 testcase warn_on
TARGET = operator-direct-tests
SOURCES += direct_tests.cpp DirectPanel.cpp WorkspaceView.cpp TraceRecorder.cpp ../desktop/DesktopShell.cpp ../desktop/DesktopBridge.cpp
HEADERS += DirectPanel.h WorkspaceView.h TraceRecorder.h ../desktop/DesktopShell.h ../desktop/DesktopBridge.h
RESOURCES += resources.qrc
