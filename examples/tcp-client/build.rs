fn main() -> Result<(), Box<dyn std::error::Error>> {
    protobuf_codegen_pure::Codegen::new()
        .out_dir("src/schema")
        .inputs(&["schema/hello_world.proto"])
        .include("schema")
        .run()
        .expect("Codegen failed.");
    Ok(())
}
