import QtQuick 2.15
import QtQuick.Controls 2.15
import QtQuick.Layouts 1.15
import VitalSigns 
// Top-level dashboard window. Expects a `dashboard` context property exposed
// from Rust (see ../src/qt/bridge.rs) with properties:
//   heartRate (int), spo2 (real), temperature (real), ecgSample (real),
//   alarmLevel (int: 0=Normal,1=Warning,2=Critical), patientId (string)
// and a method `sendConfigUpdate(hrLow, hrHigh, spo2Low, tempLow, tempHigh)`.
ApplicationWindow {
    id: root
    visible: true
    // The reTerminal's display is 1280x720 (5" touchscreen). Sized to match;
    // adjust if you're testing on a different display.
    width: 1280
    height: 720
    title: "Vital Signs Monitor Simulator"
    color: "#0d1117"
    
    Dashboard {
      id: dashboard
      Component.onCompleted: dashboard.initialize() 
    }

    ColumnLayout {
  
        anchors.fill: parent
        anchors.margins: 16
        spacing: 12

        AlarmBanner {
            Layout.fillWidth: true
            Layout.preferredHeight: 64
            level: dashboard.alarmLevel
        }

        RowLayout {
            Layout.fillWidth: true
            Layout.fillHeight: true
            spacing: 16

            Waveform {
                Layout.fillWidth: true
                Layout.fillHeight: true
                Layout.preferredWidth: parent.width * 0.65
                sample: dashboard.ecgSample
            }

            ColumnLayout {
                Layout.preferredWidth: parent.width * 0.35
                Layout.fillHeight: true
                spacing: 16

                VitalsPanel {
                    Layout.fillWidth: true
                    heartRate: dashboard.heartRate
                    spo2: dashboard.spo2
                    temperature: dashboard.temperature
                }

                SettingsPanel {
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    patientId: dashboard.patientId
                    onApply: function(hrLow, hrHigh, spo2Low, tempLow, tempHigh) {
                        dashboard.sendConfigUpdate(hrLow, hrHigh, spo2Low, tempLow, tempHigh)
                    }
                }
            }
        }
    }
}
