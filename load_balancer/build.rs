use std::env;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let proto_file = env::var("PROTO_PATH").unwrap_or("../proto/word_counter.proto".to_string());
    let proto_file_path = Path::new(&proto_file);
    let proto_path = proto_file_path.parent().expect("Path has no parent");
    tonic_build::configure()
        .build_server(false)
        .out_dir("src/generated")
        .type_attribute("WordCountResponse", "#[derive(serde::Serialize, serde::Deserialize)]")
        .type_attribute("WordCountRequest", "#[derive(serde::Serialize, serde::Deserialize)]")
        .compile_protos(&[proto_file_path], &[proto_path])?;
    Ok(())
}