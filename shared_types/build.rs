use crux_core::typegen::TypeGen;
use shared::{
    CounterApp,
    capabilities::{http::ServerConnectionState, navigation::Screen},
    features::{
        data::DataRequest,
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
    typegen.register_type::<DataRequest>()?;
    typegen.register_type::<PlaybackPosition>()?;

    // Domain
    typegen.register_type::<domain::Media>()?;
    typegen.register_type::<domain::MediaMetaData>()?;
    typegen.register_type::<domain::MediaContent>()?;
    typegen.register_type::<domain::Download>()?;
    typegen.register_type::<domain::DownloadState>()?;
    typegen.register_type::<domain::LanguageCode>()?;
    typegen.register_type::<domain::series::EpisodeIdentifier>()?;

    let output_root = PathBuf::from("./generated");
    typegen.swift("SharedTypes", output_root.join("swift"))?;
    typegen.java("com.crux.example.simple_counter", output_root.join("java"))?;

    Ok(())
}
