import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// A labeled slider with a live value readout, e.g. "HR Warning Low: 50 bpm".
ColumnLayout {
    property string label: ""
    property string unit: ""
    property real from: 0
    property real to: 100
    property real value: 50
    spacing: 2

    RowLayout {
        Layout.fillWidth: true
        Text {
            text: label
            color: "#c9d1d9"
            font.pixelSize: 12
            Layout.fillWidth: true
        }
        Text {
            text: slider.value.toFixed(1) + " " + unit
            color: "#39ff6a"
            font.pixelSize: 12
        }
    }

    Slider {
        id: slider
        Layout.fillWidth: true
        from: parent.from
        to: parent.to
        value: parent.value
        onValueChanged: parent.value = value
    }
}
