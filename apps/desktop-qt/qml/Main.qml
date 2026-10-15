import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    visible: true
    width: 1100
    height: 720
    title: "Intellectual Workbench — UI shell only"
    RowLayout {
        anchors.fill: parent
        Rectangle { Layout.preferredWidth: 220; Layout.fillHeight: true; color: "#29313c"; Text { text: "Pinned apps (stub)"; color: "white"; anchors.centerIn: parent } }
        Rectangle { Layout.fillWidth: true; Layout.fillHeight: true; color: "#f2f3f5"; Text { text: "Operator (not connected)"; anchors.centerIn: parent } }
        Rectangle { Layout.preferredWidth: 360; Layout.fillHeight: true; color: "#e4e8ed"; Text { text: "Canvas (not connected)"; anchors.centerIn: parent } }
    }
}
