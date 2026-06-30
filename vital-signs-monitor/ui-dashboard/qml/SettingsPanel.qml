import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15

// Threshold sliders + patient ID field. Emits `apply` with the five
// threshold values when the operator taps "Apply", which main.qml wires
// to dashboard.sendConfigUpdate(...) -> a ConfigUpdate frame to control-core.
Rectangle {
    id: root
    property string patientId: ""
    signal apply(int hrLow, int hrHigh, int spo2Low, int tempLow, int tempHigh)

    color: "#11161d"
    radius: 10
    border.color: "#262d38"
    border.width: 1

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 16
        spacing: 10

        Text {
            text: "SETTINGS"
            color: "#8b949e"
            font.pixelSize: 12
            font.bold: true
        }

        RowLayout {
            Layout.fillWidth: true
            Text { text: "Patient ID:"; color: "#c9d1d9"; font.pixelSize: 13 }
            TextField {
                id: patientIdField
                Layout.fillWidth: true
                text: root.patientId
                color: "#c9d1d9"
                placeholderText: "e.g. P-00123"
            }
        }

        ThresholdSlider {
            id: hrLowSlider
            Layout.fillWidth: true
            label: "HR Warning Low"
            unit: "bpm"
            from: 30; to: 80; value: 50
        }
        ThresholdSlider {
            id: hrHighSlider
            Layout.fillWidth: true
            label: "HR Warning High"
            unit: "bpm"
            from: 90; to: 180; value: 110
        }
        ThresholdSlider {
            id: spo2LowSlider
            Layout.fillWidth: true
            label: "SpO2 Warning Low"
            unit: "%"
            from: 80; to: 95; value: 92
        }
        ThresholdSlider {
            id: tempLowSlider
            Layout.fillWidth: true
            label: "Temp Warning Low"
            unit: "°C"
            from: 34; to: 37; value: 36
        }
        ThresholdSlider {
            id: tempHighSlider
            Layout.fillWidth: true
            label: "Temp Warning High"
            unit: "°C"
            from: 38; to: 41; value: 38.5
        }

        Button {
            Layout.fillWidth: true
            text: "Apply Thresholds"
            onClicked: root.apply(
                Math.round(hrLowSlider.value),
                Math.round(hrHighSlider.value),
                Math.round(spo2LowSlider.value * 10),
                Math.round(tempLowSlider.value * 100),
                Math.round(tempHighSlider.value * 100)
            )
        }
    }
}
