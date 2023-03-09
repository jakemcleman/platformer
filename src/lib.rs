mod actions;
mod loading;
mod player;
mod actor;
mod camera;
mod door;
mod pickup;
mod sprite_anim;
mod ui_events;
mod world;

use crate::actions::ActionsPlugin;
use crate::camera::CameraPlugin;
use crate::loading::LoadingPlugin;
use crate::pickup::PickupPlugin;
use crate::player::PlayerPlugin;
use crate::world::WorldPlugin;

use actor::ActorPlugin;
use bevy::app::App;
#[cfg(debug_assertions)]
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_rapier2d::render::RapierDebugRenderPlugin;
use sprite_anim::SpriteAnimationPlugin;
use ui_events::UiEventPlugin;

// This example game uses States to separate logic
// See https://bevy-cheatbook.github.io/programming/states.html
// Or https://github.com/bevyengine/bevy/blob/main/examples/ecs/state.rs
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
enum GameState {
    // During the loading State the LoadingPlugin will load our assets
    #[default]
    Loading,
    // During this State the actual game logic is executed
    Playing,
    // Here the menu is drawn and waiting for player interaction
    Menu,
    // To be used over the playing state
    Paused,
    // Shows level selection menu
    LevelSelect,
    // Shows win screen, links back to main menu
    WinScreen,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_state::<GameState>()
            .add_plugins(WorldPlugin)
            .add_plugins(LoadingPlugin)
            .add_plugins(UiEventPlugin)
            .add_plugins(ActionsPlugin)
            .add_plugins(PlayerPlugin)
            .add_plugins(PickupPlugin)
            .add_plugins(ActorPlugin)
            .add_plugins(SpriteAnimationPlugin)
            .add_plugins(CameraPlugin);

        #[cfg(debug_assertions)]
        {
            app.add_plugins(FrameTimeDiagnosticsPlugin::default())
                .add_plugins(LogDiagnosticsPlugin::default())
                .add_plugins(RapierDebugRenderPlugin::default());
        }
    }
}
