import QtQuick 2.15

// Color-coded banner reflecting the current AlarmLevel from control-core.
// level: 0 = Normal (green), 1 = Warning (amber), 2 = Critical (red, flashing).
Rectangle {
    id: root
    property int level: 0

    radius: 8
    color: level === 2 ? "#7a1f1f" : (level === 1 ? "#7a5c1f" : "#1f3a26")
    border.color: level === 2 ? "#ff4d4d" : (level === 1 ? "#ffb84d" : "#39ff6a")
    border.width: 2

    // Critical alarms flash to draw attention, matching real bedside
    // monitor behavior for the highest-priority alarm state.
    SequentialAnimation on opacity {
        running: level === 2
        loops: Animation.Infinite
        NumberAnimation { from: 1.0; to: 0.55; duration: 500 }
        NumberAnimation { from: 0.55; to: 1.0; duration: 500 }
    }

    Text {
        anchors.centerIn: parent
        text: level === 2 ? "CRITICAL ALARM" : (level === 1 ? "WARNING" : "NORMAL — ALL VITALS WITHIN RANGE")
        color: "#ffffff"
        font.pixelSize: 20
        font.bold: true
    }
}
