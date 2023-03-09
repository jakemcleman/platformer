use bevy::prelude::*;

use crate::actions::game_control::{get_movement, GameControl};

use self::game_control::get_gamepad_movement;

mod game_control;

pub struct ActionsPlugin;

// This plugin listens for keyboard input and converts the input into Actions
// Actions can then be used as a resource in other systems to act on the player input.
impl Plugin for ActionsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Actions>()
            .add_systems(Update, set_movement_actions)
            .add_systems(Update, set_pause_actions)
            .add_systems(Update, set_back_actions)
        ;
    }
}

#[derive(Default, Resource)]
pub struct Actions {
    pub player_movement: [Vec2; 2],
    pub jump: [bool; 2],
    pub action: [bool; 2],
    pub pause: bool,
    pub back: bool,
}

pub fn set_pause_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Gamepads>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    actions.pause = keyboard_input.just_pressed(KeyCode::Escape);

    for gamepad in gamepad_input.iter() {
        if actions.pause {
            break;
        }

        actions.pause =
            gamepad_buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::Start));
    }
}

pub fn set_back_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Gamepads>,
    gamepad_buttons: Res<Input<GamepadButton>>,
) {
    actions.back = keyboard_input.just_pressed(KeyCode::Escape);

    for gamepad in gamepad_input.iter() {
        if actions.back {
            break;
        }

        actions.back =
            gamepad_buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::East));
    }
}

pub fn set_movement_actions(
    mut actions: ResMut<Actions>,
    keyboard_input: Res<Input<KeyCode>>,
    gamepad_input: Res<Gamepads>,
    gamepad_buttons: Res<Input<GamepadButton>>,
    gamepad_axes: Res<Axis<GamepadAxis>>,
) {
    let gamepad_movement = get_gamepad_movement(&gamepad_input, &gamepad_buttons, &gamepad_axes);

    let keyboard_movement = Vec2::new(
        get_movement(GameControl::Right, &keyboard_input)
            - get_movement(GameControl::Left, &keyboard_input),
        get_movement(GameControl::Up, &keyboard_input)
            - get_movement(GameControl::Down, &keyboard_input),
    );

    if keyboard_movement != Vec2::ZERO {
        actions.player_movement[0] = keyboard_movement.normalize();
    } else if gamepad_movement[0].length_squared() > 0.1 {
        actions.player_movement[0] = gamepad_movement[0].normalize();
    } else {
        actions.player_movement[0] = Vec2::ZERO;
    }
    
    if gamepad_movement[1].length_squared() > 0.1 {
        actions.player_movement[1] = gamepad_movement[1].normalize();
    } else {
        actions.player_movement[1] = Vec2::ZERO;
    }

    actions.jump[0] = keyboard_input.pressed(KeyCode::Space) || keyboard_movement.y > 0.;
    actions.jump[1] = false;

    for gamepad in gamepad_input.iter() {
        
        if gamepad.id > 1 || actions.jump[gamepad.id] {
            break;
        }
        actions.jump[gamepad.id] = actions.jump[gamepad.id]
            || gamepad_buttons.pressed(GamepadButton::new(gamepad, GamepadButtonType::South));
    }

    actions.action[0] = keyboard_input.just_pressed(KeyCode::Q)
        || keyboard_input.just_pressed(KeyCode::E);
    
    actions.action[1] = keyboard_input.just_pressed(KeyCode::M);

    for gamepad in gamepad_input.iter() {
        if gamepad.id > 1 || actions.action[gamepad.id] {
            break;
        }
        actions.action[gamepad.id] = actions.action[gamepad.id]
            || gamepad_buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::West))
            || gamepad_buttons.just_pressed(GamepadButton::new(gamepad, GamepadButtonType::East));
    }
}
