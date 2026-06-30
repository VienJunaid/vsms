import QtQuick 2.15
import QtQuick.Layouts 1.15

// Large numeric readouts for HR / SpO2 / Temp, styled like a real bedside
// monitor's vitals tile (big number, small unit/label underneath).
Rectangle {
    id: root
    property int heartRate: 0
    property real spo2: 0.0
    property real temperature: 0.0

    color: "#11161d"
    radius: 10
    border.color: "#262d38"
    border.width: 1
    implicitHeight: 220

    GridLayout {
        anchors.fill: parent
        anchors.margins: 16
        columns: 3
        columnSpacing: 12

        VitalTile {
            Layout.fillWidth: true
            Layout.fillHeight: true
            label: "HEART RATE"
            value: heartRate > 0 ? heartRate.toString() : "--"
            unit: "bpm"
            accentColor: "#39ff6a"
        }

        VitalTile {
            Layout.fillWidth: true
            Layout.fillHeight: true
            label: "SpO2"
            value: spo2.toFixed(1)
            unit: "%"
            accentColor: "#37c4ff"
        }

        VitalTile {
            Layout.fillWidth: true
            Layout.fillHeight: true
            label: "TEMP"
            value: temperature.toFixed(2)
            unit: "°C"
            accentColor: "#ffb84d"
        }
    }
}
