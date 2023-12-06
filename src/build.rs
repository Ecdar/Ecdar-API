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
        .type_attribute("ProjectInfo", "#[derive(sea_orm::FromQueryResult)]")
        .type_attribute("Error", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "ComponentsNotInCache",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Success", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "ModelFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ParsingError",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ReachabilityPath",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "RefinementFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ReachabilityFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ImplementationFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "DeterminismFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ConsistencyFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Path", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "ActionFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "StateAction",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "RefinementStateFailure",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ActionSet",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "Decision",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Edge", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "ComponentInstance",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "Disjunction",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "LocationTree",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("State", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "BinaryLocationOperator",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "LeafLocation",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "Conjunction",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "Constraint",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute("Clock", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute(
            "ZeroClock",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "SystemClock",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .enum_attribute("clock", "#[derive(serde::Serialize, serde::Deserialize)]")
        .enum_attribute(
            "node_type",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .type_attribute(
            "ComponentClock",
            "#[derive(serde::Serialize, serde::Deserialize)]",
        )
        .enum_attribute("failure", "#[derive(serde::Serialize, serde::Deserialize)]")
        .enum_attribute("rep", "#[derive(serde::Serialize, serde::Deserialize)]")
        .enum_attribute("result", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile(&["Ecdar-ProtoBuf/services.proto"], &["Ecdar-ProtoBuf/"])
        .unwrap();

    // Tell cargo to invalidate the crate when the protobuf repository changes
    println!("cargo:rerun-if-changed=Ecdar-ProtoBuf");
}
