fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto");
    let descriptors = protox::compile(["api/v1/health.proto", "ui/v1/shell.proto"], ["../proto"])?;
    prost_build::Config::new().compile_fds(descriptors)?;
    body_tokens()
}

/// Message bodies take their text colour from the app's design tokens.
fn body_tokens() -> Result<(), Box<dyn std::error::Error>> {
    const TOKENS: &str = "../design/tokens.json";
    println!("cargo:rerun-if-changed={TOKENS}");
    let tokens: serde_json::Value = serde_json::from_str(&std::fs::read_to_string(TOKENS)?)?;
    let text = tokens["color"]["tokens"]
        .as_array()
        .and_then(|all| all.iter().find(|t| t["name"] == "text"))
        .ok_or("design/tokens.json has no text colour")?;
    let value = |theme: &str| {
        text["value"][theme]
            .as_str()
            .ok_or(format!("the text colour has no {theme} value"))
    };
    let out = std::path::Path::new(&std::env::var("OUT_DIR")?).join("body_tokens.rs");
    std::fs::write(
        out,
        format!(
            "const TEXT_DARK: &str = {:?};\nconst TEXT_LIGHT: &str = {:?};\n",
            value("dark")?,
            value("light")?
        ),
    )?;
    Ok(())
}
