// disable console on windows for release builds
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use bevy::{
    prelude::*,
    window::{PresentMode, WindowTheme},
    DefaultPlugins,
};
use platformer_game::GamePlugin;

fn main() {
    App::new()
        .insert_resource(Msaa::Sample4)
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .add_plugins(
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        resolution: (1200., 800.).into(),
                        title: "Platformer Game".to_string(),
                        present_mode: PresentMode::AutoVsync,
                        // Tells wasm to resize the window according to the available canvas
                        fit_canvas_to_parent: true,
                        // Tells wasm not to override default event handling, like F5, Ctrl+R etc.
                        prevent_default_event_handling: false,
                        window_theme: Some(WindowTheme::Dark),
                        ..Default::default()
                    }),
                    ..default()
                })
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin {
                    ..Default::default()
                }),
        )
        .add_plugins(GamePlugin)
        .run();
}
