import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

ApplicationWindow {
    id: window
    visible: true
    width: 680
    height: 720
    title: bridge.title || "Semantic Workbench"
    color: bridge.darkMode ? "#20242b" : "#f4f5f7"
    property color foreground: bridge.darkMode ? "#f5f7fa" : "#20242b"
    ScrollView {
        anchors.fill: parent
        anchors.margins: 20
        ColumnLayout {
            width: window.width - 60
            spacing: 14
            Label {
                Layout.fillWidth: true
                wrapMode: Text.WrapAnywhere
                text: bridge.title
                textFormat: Text.PlainText
                color: window.foreground
                font.pixelSize: 24 * bridge.textScale
                Accessible.role: Accessible.Heading
                Accessible.name: text
            }
            Repeater {
                // Dispose old view-owned editors while selection is pending, even when
                // the next room has byte-identical control metadata.
                model: bridge.ready ? bridge.controls : []
                delegate: ColumnLayout {
                    id: row
                    required property var modelData
                    Layout.fillWidth: true
                    Label {
                        Layout.fillWidth: true
                        wrapMode: Text.WrapAnywhere
                        text: row.modelData.label + (row.modelData.unit ? " (" + row.modelData.unit + ")" : "")
                        textFormat: Text.PlainText
                        color: window.foreground
                        font.pixelSize: 16 * bridge.textScale
                        Accessible.role: Accessible.StaticText
                        Accessible.name: row.modelData.label
                    }
                    Loader {
                        Layout.fillWidth: true
                        sourceComponent: row.modelData.type === "readout" ? outputControl : inputControl
                        Component {
                            id: inputControl
                            TextField {
                                id: editor
                                property string ownerEpoch: ""
                                objectName: "input_" + row.modelData.id
                                Accessible.role: Accessible.EditableText
                                Accessible.name: row.modelData.label
                                Accessible.description: bridge.error
                                activeFocusOnTab: true
                                font.pixelSize: 18 * bridge.textScale
                                selectByMouse: true
                                maximumLength: 512
                                Component.onCompleted: { ownerEpoch = bridge.viewEpoch; text = bridge.committedValue(row.modelData.id) }
                                onTextEdited: bridge.setLocalDraft(row.modelData.id, text, ownerEpoch)
                                // Focus changes are presentation events, not input changes.
                                onEditingFinished: {
                                    if (bridge.hasDraft(row.modelData.id))
                                        bridge.edit(row.modelData.id, text, ownerEpoch)
                                }
                                Connections {
                                    target: bridge
                                    function onValuesChanged() {
                                        if (editor.ownerEpoch === bridge.viewEpoch && !editor.activeFocus && !bridge.hasDraft(row.modelData.id))
                                            editor.text = bridge.committedValue(row.modelData.id)
                                    }
                                }
                            }
                        }
                        Component {
                            id: outputControl
                            Label {
                                id: readout
                                width: parent.width
                                wrapMode: Text.WrapAnywhere
                                objectName: "output_" + row.modelData.id
                                Component.onCompleted: text = bridge.committedValue(row.modelData.id)
                                Connections {
                                    target: bridge
                                    function onValuesChanged() { readout.text = bridge.committedValue(row.modelData.id) }
                                }
                                textFormat: Text.PlainText
                                color: window.foreground
                                font.pixelSize: 22 * bridge.textScale
                                Accessible.role: Accessible.StaticText
                                Accessible.name: row.modelData.label
                                Accessible.description: bridge.error
                            }
                        }
                    }
                }
            }
            Label {
                objectName: "diagnostic"
                text: bridge.error
                visible: text.length > 0
                textFormat: Text.PlainText
                wrapMode: Text.Wrap
                color: bridge.darkMode ? "#ffaaaa" : "#a00000"
                font.pixelSize: 16 * bridge.textScale
                Accessible.role: Accessible.AlertMessage
                Accessible.name: text
            }
            Button {
                objectName: "applyDraft"
                text: "Apply draft"
                enabled: bridge.ready
                font.pixelSize: 16 * bridge.textScale
                Accessible.name: text
                property string pressedEpoch: ""
                onPressed: pressedEpoch = bridge.viewEpoch
                onClicked: bridge.apply(pressedEpoch)
            }
        }
    }
}
