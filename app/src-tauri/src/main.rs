#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;

use asiobridge_core::{AudioEngine, Channel, ChannelId, Rack, RackId};
use std::sync::Mutex;
use tauri::Manager;
use tracing::info;

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("asiobridge=info".parse().unwrap()),
        )
        .init();

    let engine = AudioEngine::new(44100, 24, 2);

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_store::Builder::new().build())
        .manage(Mutex::new(engine))
        .setup(|app| {
            let engine = app.state::<Mutex<AudioEngine>>();
            let mut engine = engine.lock().unwrap();

            // Initialize default racks
            for rack_id in [
                "asio-driver-in",
                "asio-driver-out",
                "asio-host-in",
                "network-in",
                "network-out",
                "looper-in",
                "looper-out",
                "wdm-in",
                "mix-out",
            ] {
                let rack = match rack_id {
                    "asio-driver-in" => Rack::new(
                        RackId::AsioDriverIn,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("IN {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: true,
                            })
                            .collect(),
                    ),
                    "asio-driver-out" => Rack::new(
                        RackId::AsioDriverOutMix,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("OUT {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: true,
                            })
                            .collect(),
                    ),
                    "asio-host-in" => Rack::new(
                        RackId::AsioHostInMix,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("HOST {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: true,
                            })
                            .collect(),
                    ),
                    "network-in" => Rack::new(
                        RackId::NetworkIn,
                        (0..4)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("NET-IN {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: false,
                            })
                            .collect(),
                    ),
                    "network-out" => Rack::new(
                        RackId::NetworkOut,
                        (0..4)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("NET-OUT {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: false,
                            })
                            .collect(),
                    ),
                    "looper-in" => Rack::new(
                        RackId::LooperIn,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("LOOPER-IN {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: false,
                            })
                            .collect(),
                    ),
                    "looper-out" => Rack::new(
                        RackId::LooperOut,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("LOOPER-OUT {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: false,
                            })
                            .collect(),
                    ),
                    "wdm-in" => Rack::new(
                        RackId::WdmIn,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("WDM {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: false,
                            })
                            .collect(),
                    ),
                    "mix-out" => Rack::new(
                        RackId::MixOut,
                        (0..8)
                            .map(|i| Channel {
                                id: ChannelId(i as u32),
                                name: format!("MIX {}", i + 1),
                                sample_rate: 44100,
                                bit_depth: 24,
                                is_active: true,
                            })
                            .collect(),
                    ),
                    _ => unreachable!(),
                };
                engine.add_rack(rack);
            }

            info!("AsioBridge initialized with {} racks", engine.get_racks().len());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_racks,
            commands::get_connections,
            commands::start_engine,
            commands::stop_engine,
            commands::get_engine_config,
            commands::set_sample_rate,
            commands::save_profile,
            commands::load_profile,
            commands::add_connection,
            commands::remove_connection,
        ])
        .run(tauri::generate_context!())
        .expect("error while running AsioBridge");
}
