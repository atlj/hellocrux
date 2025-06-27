use crux_core::typegen::TypeGen;
use domain::Media;
use shared::{
    CounterApp,
    capabilities::{http::HttpRequestState, navigation::Screen},
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut typegen = TypeGen::new();
    typegen.register_app::<CounterApp>()?;
    typegen.register_type::<Screen>()?;
    typegen.register_type::<HttpRequestState>()?;
    typegen.register_type::<Media>()?;

    let output_root = PathBuf::from("./generated");
    typegen.swift("SharedTypes", output_root.join("swift"))?;
    typegen.java("com.crux.example.simple_counter", output_root.join("java"))?;

    Ok(())
}
