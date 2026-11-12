fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto");
    let descriptors = protox::compile(["api/v1/health.proto", "ui/v1/shell.proto"], ["../proto"])?;
    prost_build::Config::new().compile_fds(descriptors)?;
    Ok(())
}
