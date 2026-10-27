QT += core gui widgets qml quick quickcontrols2 testlib
CONFIG += c++17 warn_on
TARGET = workbench-operator
SOURCES += main.cpp OperatorPanel.cpp OperatorDriver.cpp ../desktop/DesktopShell.cpp ../desktop/DesktopBridge.cpp
HEADERS += OperatorPanel.h OperatorDriver.h ../desktop/DesktopShell.h ../desktop/DesktopBridge.h
RESOURCES += ../resources.qrc
