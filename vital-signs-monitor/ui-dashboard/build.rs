// build.rs

// This file tells Cargo how to compile the cxx-qt bridge.
// Without it, Rust has no idea that bridge.rs contains C++/Qt glue code that needs special build
// steps.

fn main() {
    cxx_qt_build::CxxQtBuilder::new()
        .qt_module("Core") // tells cxx-qt which Qt libraries to link against. Core is always
        .qt_module("Qml") // needed; Qml and Quick are needed because UI uses QML and QtQuick
        .qt_module("Quick") // controls
        .qml_module(cxx_qt_build::QmlModule { // bundles all the QML files into a compiled Qt
                                              // resource under the VitalSigns URI 
            uri: "VitalSigns",
            rust_files: &["src/qt/bridge.rs"],
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
