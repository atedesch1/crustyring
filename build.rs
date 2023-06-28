fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("./proto/registry.proto")?;
    tonic_build::compile_protos("./proto/dht.proto")?;
    Ok(())
}
