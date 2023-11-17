fn main() {
    tonic_build::compile_protos("Ecdar-ProtoBuf/services.proto").unwrap();
    // Tell cargo to invalidate the crate when the protobuf repository changes
    println!("cargo:rerun-if-changed=Ecdar-ProtoBuf");
}
