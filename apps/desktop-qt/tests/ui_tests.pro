QT += core gui qml quick quickcontrols2 testlib
CONFIG += c++17 testcase warn_on
TARGET = ui-conformance
SOURCES += UiConformance.cpp ../src/UiBridge.cpp
HEADERS += ../src/UiBridge.h
INCLUDEPATH += ../src
RESOURCES += ../resources.qrc
