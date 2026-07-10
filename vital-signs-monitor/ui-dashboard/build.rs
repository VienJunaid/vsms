// build.rs

// This file tells Cargo how to compile the cxx-qt bridge.
// Without it, Rust has no idea that bridge.rs contains C++/Qt glue code that needs special build
// steps.

fn main() {
    let qt_modules = vec!["Core", "Qml", "Quick"];

    cxx_qt_build::CxxQtBuilder::new()
        .qt_modules(qt_modules) // tells cxx-qt which Qt libraries to link against. Core is always
                                // needed Qml and Quick are needed because UI uses QMl and QtQuick
                                // controls 
        .rust_bridge("src/qt/bridge") // points cxx-qt to the bridge file to generate C++ glue code
                                      // for the cxx_qt::bridge macro 
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
