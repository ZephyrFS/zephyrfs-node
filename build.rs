fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate protobuf code for node service (existing)
    tonic_build::configure()
        .build_server(true)
        .build_client(true)
        .compile(
            &["../zephyrfs-proto/protobuff/node.proto"],
            &["../zephyrfs-proto/protobuff"],
        )?;

    // Generate protobuf code for coordinator service (new)
    tonic_build::configure()
        .build_server(false) // We only need the client
        .build_client(true)
        .compile(
            &["../zephyrfs-proto/protobuff/coordinator.proto"],
            &["../zephyrfs-proto/protobuff"],
        )?;

    println!("cargo:rerun-if-changed=../zephyrfs-proto/protobuff/coordinator.proto");
    println!("cargo:rerun-if-changed=../zephyrfs-proto/protobuff/node.proto");
    Ok(())
}