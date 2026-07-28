fn main() {
    #[cfg(feature = "qt-ui")]
    cxx_qt_build::CxxQtBuilder::new()
        .qt_module("Core")
        .qt_module("Qml")
        .qt_module("Quick")
        .qml_module(cxx_qt_build::QmlModule {
            uri: "VitalSigns",
            rust_files: &["src/bridge.rs"],
            qml_files: &[
                "qml/main.qml",
                "qml/AlarmBanner.qml",
                "qml/VitalsPanel.qml",
                "qml/VitalTile.qml",
                "qml/Waveform.qml",
                "qml/SettingsPanel.qml",
                "qml/ThresholdSlider.qml",
            ],
            ..Default::default()
        })
        .build();
}
