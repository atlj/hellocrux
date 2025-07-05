use crux_core::typegen::TypeGen;
use domain::{Media, MediaContent, MediaMetaData};
use shared::{
    CounterApp, PlayEvent,
    capabilities::{http::HttpRequestState, navigation::Screen},
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut typegen = TypeGen::new();
    typegen.register_app::<CounterApp>()?;
    typegen.register_type::<Screen>()?;
    typegen.register_type::<HttpRequestState>()?;
    typegen.register_type::<PlayEvent>()?;

    // Domain
    typegen.register_type::<Media>()?;
    typegen.register_type::<MediaMetaData>()?;
    typegen.register_type::<MediaContent>()?;

    let output_root = PathBuf::from("./generated");
    typegen.swift("SharedTypes", output_root.join("swift"))?;
    typegen.java("com.crux.example.simple_counter", output_root.join("java"))?;

    Ok(())
}
