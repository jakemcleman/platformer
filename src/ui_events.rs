use bevy::{app::AppExit, prelude::*};
// use bevy_pkv::PkvStore;
use crate::GameState;

#[derive(Debug, Event)]
pub enum UiEvent {
    _QuitGame,
    _NewGame,
    _LoadGame,
}

pub struct UiEventPlugin;

impl Plugin for UiEventPlugin {
    fn build(&self, app: &mut App) {
        app
            // Register a event that can be called from the action handler
            .add_event::<UiEvent>()
            .add_systems(Update, event_reader);
    }
}

/// This reacts to actions fired from UI with custom bevy resources or eventwriters or queries.
fn event_reader(
    mut event_reader: EventReader<UiEvent>,
    mut exit: EventWriter<AppExit>,
    mut next_state:  ResMut<NextState<GameState>>,
    // mut pkv: ResMut<PkvStore>,
) {
    for event in event_reader.iter() {
        match event {
            UiEvent::_QuitGame => exit.send(AppExit),
            UiEvent::_NewGame => next_state.set(GameState::Playing),
            UiEvent::_LoadGame => next_state.set(GameState::Playing),
        }
    }
}
