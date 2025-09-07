fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=../proto/analytics.proto");
    println!("cargo:rerun-if-changed=../proto/trade.proto");

    prost_build::compile_protos(
        &["../proto/analytics.proto", "../proto/trade.proto"],
        &["../proto/"],
    )?;
    Ok(())
}
