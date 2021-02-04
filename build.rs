use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if Ok("dev".to_owned()) == env::var("PROFILE") {
        protobuf_codegen_pure::Codegen::new()
            .out_dir("src/schema")
            .inputs(&["schema/message.proto"])
            .include("schema")
            .run()
            .expect("Codegen failed.");
    }

    Ok(())
}
