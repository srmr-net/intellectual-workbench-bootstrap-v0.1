QT += core gui widgets qml quick quickcontrols2 testlib
CONFIG += c++17 warn_on
TARGET = workbench-desktop
SOURCES += main.cpp DesktopShell.cpp DesktopBridge.cpp ConformanceDriver.cpp
HEADERS += DesktopShell.h DesktopBridge.h ConformanceDriver.h
RESOURCES += ../resources.qrc
