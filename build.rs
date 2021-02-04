fn main() -> Result<(), Box<dyn std::error::Error>> {
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/schema")
        .inputs(&["schema/message.proto"])
        .include("schema")
        .run()
        .expect("Codegen failed.");

    println!("cargo:rerun-if-changed=build.rs");

    Ok(())
}
