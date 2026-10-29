QT += core gui widgets qml quick quickcontrols2 testlib
CONFIG += c++17 testcase warn_on
TARGET = operator-direct-tests
SOURCES += direct_tests.cpp DirectPanel.cpp ../desktop/DesktopShell.cpp ../desktop/DesktopBridge.cpp
HEADERS += DirectPanel.h ../desktop/DesktopShell.h ../desktop/DesktopBridge.h
RESOURCES += ../vslice-003/resources.qrc
