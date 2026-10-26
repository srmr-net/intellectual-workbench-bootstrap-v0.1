QT += core gui qml quick quickcontrols2
CONFIG += c++17 warn_on
TARGET = workbench-ui
SOURCES += src/main.cpp src/UiBridge.cpp
HEADERS += src/UiBridge.h
RESOURCES += resources.qrc
