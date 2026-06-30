import QtQuick 2.15

// Scrolling ECG trace, rendered on a Canvas as a classic "EKG strip".
// `sample` is pushed in (millivolts) on every VitalsSample frame from
// control-core; we ring-buffer the last N samples and redraw.
Item {
    id: root
    property real sample: 0.0

    // Roughly 6 seconds of trace at the UI's effective refresh rate.
    // Note: control-core samples at 250Hz internally but only pushes
    // ui-relevant updates at the rate the socket delivers them, so this
    // is intentionally a display buffer, not the raw 250Hz stream.
    property int bufferSize: 600
    property var history: []

    onSampleChanged: {
        history.push(sample)
        if (history.length > bufferSize) {
            history.shift()
        }
        canvas.requestPaint()
    }

    Rectangle {
        anchors.fill: parent
        color: "#000000"
        border.color: "#1f2630"
        border.width: 1
        radius: 6
    }

    Canvas {
        id: canvas
        anchors.fill: parent
        anchors.margins: 8

        onPaint: {
            var ctx = getContext("2d")
            ctx.clearRect(0, 0, width, height)

            // Grid, classic EKG-strip green-on-black style.
            ctx.strokeStyle = "#163a1f"
            ctx.lineWidth = 1
            var gridSpacing = 24
            for (var gx = 0; gx < width; gx += gridSpacing) {
                ctx.beginPath()
                ctx.moveTo(gx, 0)
                ctx.lineTo(gx, height)
                ctx.stroke()
            }
            for (var gy = 0; gy < height; gy += gridSpacing) {
                ctx.beginPath()
                ctx.moveTo(0, gy)
                ctx.lineTo(width, gy)
                ctx.stroke()
            }

            if (history.length < 2) return

            ctx.strokeStyle = "#39ff6a"
            ctx.lineWidth = 2
            ctx.beginPath()

            var midY = height / 2
            // mV-to-pixel scale: tuned for the ~ -0.2..1.3mV range our
            // synthetic QRS complex produces (see control-core/src/signals.rs).
            var scale = height / 3.0
            var stepX = width / bufferSize

            for (var i = 0; i < history.length; i++) {
                var x = i * stepX
                var y = midY - history[i] * scale
                if (i === 0) {
                    ctx.moveTo(x, y)
                } else {
                    ctx.lineTo(x, y)
                }
            }
            ctx.stroke()
        }
    }

    Text {
        anchors.top: parent.top
        anchors.left: parent.left
        anchors.margins: 12
        text: "ECG"
        color: "#39ff6a"
        font.pixelSize: 14
        font.bold: true
    }
}
