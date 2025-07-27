use crux_core::typegen::TypeGen;
use domain::{Media, MediaContent, MediaMetaData};
use shared::{
    CounterApp,
    capabilities::{http::ServerConnectionState, navigation::Screen},
    features::{
        playback::{PlayEvent, PlaybackPosition},
        server_communication::ServerCommunicationEvent,
    },
};
use std::path::PathBuf;

fn main() -> anyhow::Result<()> {
    println!("cargo:rerun-if-changed=../shared");

    let mut typegen = TypeGen::new();
    typegen.register_app::<CounterApp>()?;
    typegen.register_type::<Screen>()?;
    typegen.register_type::<ServerConnectionState>()?;
    typegen.register_type::<PlayEvent>()?;
    typegen.register_type::<ServerCommunicationEvent>()?;
    typegen.register_type::<PlaybackPosition>()?;

    // Domain
    typegen.register_type::<Media>()?;
    typegen.register_type::<MediaMetaData>()?;
    typegen.register_type::<MediaContent>()?;

    let output_root = PathBuf::from("./generated");
    typegen.swift("SharedTypes", output_root.join("swift"))?;
    typegen.java("com.crux.example.simple_counter", output_root.join("java"))?;

    Ok(())
}
