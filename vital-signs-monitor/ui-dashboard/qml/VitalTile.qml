import QtQuick 2.15

// A single big-number readout tile, e.g. "80 bpm" under a "HEART RATE" label.
Rectangle {
    property string label: ""
    property string value: "--"
    property string unit: ""
    property color accentColor: "#39ff6a"

    color: "#0d1117"
    radius: 8

    Column {
        anchors.centerIn: parent
        spacing: 4

        Text {
            text: label
            color: "#8b949e"
            font.pixelSize: 12
            font.bold: true
            anchors.horizontalCenter: parent.horizontalCenter
        }

        Row {
            spacing: 4
            anchors.horizontalCenter: parent.horizontalCenter
            Text {
                text: value
                color: accentColor
                font.pixelSize: 36
                font.bold: true
            }
            Text {
                text: unit
                color: accentColor
                font.pixelSize: 16
                anchors.bottom: parent.bottom
                anchors.bottomMargin: 6
            }
        }
    }
}
