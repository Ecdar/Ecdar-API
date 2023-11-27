fn main() {
    tonic_build::configure()
        .type_attribute(
            "ComponentsInfo",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "Component",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .enum_attribute("rep", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["Ecdar-ProtoBuf/services.proto"], &["Ecdar-ProtoBuf/"])
        .unwrap();

    // Tell cargo to invalidate the crate when the protobuf repository changes
    println!("cargo:rerun-if-changed=Ecdar-ProtoBuf");
}
