QT += core gui widgets qml quick quickcontrols2 testlib
CONFIG += c++17 warn_on testcase
TARGET = vs2-qt-conformance
INCLUDEPATH += ../desktop
SOURCES += Vs2Conformance.cpp ../desktop/DesktopShell.cpp ../desktop/DesktopBridge.cpp
HEADERS += ../desktop/DesktopShell.h ../desktop/DesktopBridge.h
RESOURCES += ../resources.qrc
