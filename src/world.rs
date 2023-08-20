use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;
use std::collections::{HashMap, HashSet};

use crate::GameState;

pub struct WorldPlugin;

#[derive(Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Labeled {
    pub name: String,
}

#[derive(Event)]
pub struct ReloadWorldEvent;

#[derive(Event)]
pub struct ChangeLevelEvent {
    pub index: usize,
    pub completed: bool,
    pub win_game: bool,
}

#[derive(Resource)]
pub struct LdtkImageHolder(Handle<Image>);

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(LevelSelection::Index(0))
            .add_event::<ReloadWorldEvent>()
            .add_event::<ChangeLevelEvent>()
            .add_plugins(LdtkPlugin)
            .insert_resource(LdtkSettings {
                level_background: LevelBackground::Nonexistent,
                ..default()
            })
            .add_plugins(RapierPhysicsPlugin::<NoUserData>::pixels_per_meter(100.0))
            .insert_resource(RapierConfiguration {
                gravity: Vec2::new(0.0, -2000.0),
                ..Default::default()
            })
            .add_systems(OnEnter(GameState::Playing), setup_world)
            .add_systems(OnExit(GameState::Playing), cleanup_world)
            .add_systems(Update, (switch_level, reload_level, spawn_wall_collision))
            
            .register_ldtk_entity::<crate::player::PlayerBundle>("Player")
            .register_ldtk_entity::<crate::door::DoorBundle>("Door")
            .register_ldtk_entity::<WheatBundle>("Wheat")
            .register_ldtk_int_cell::<WallBundle>(1)
            .register_ldtk_int_cell::<SpikeBundle>(2);

        #[cfg(debug_assertions)]
        {
            app.add_systems(
                Update, test_switch_level
            );
        }
    }
}

fn cleanup_world(mut commands: Commands, query: Query<Entity, Without<OrthographicProjection>>) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }
}

fn reload_level(
    mut commands: Commands,
    level_query: Query<Entity, With<Handle<LdtkLevel>>>,
    input: Res<Input<KeyCode>>,
    reload_event_listener: EventReader<ReloadWorldEvent>,
) {
    if !reload_event_listener.is_empty() || input.just_pressed(KeyCode::R) {
        println!("reloading level");
        for level_entity in &level_query {
            commands.entity(level_entity).insert(Respawn);
        }
    }
}

fn switch_level(
    mut level_selection: ResMut<LevelSelection>,
    mut change_event_listener: EventReader<ChangeLevelEvent>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for ev in change_event_listener.iter() {
        if ev.win_game {
            next_state.set(GameState::WinScreen);
        } else {
            *level_selection = LevelSelection::Index(ev.index);
        }
    }
}

fn test_switch_level(mut level_selection: ResMut<LevelSelection>, input: Res<Input<KeyCode>>) {
    if input.just_pressed(KeyCode::Key1) {
        *level_selection = LevelSelection::Index(0);
    } else if input.just_pressed(KeyCode::Key2) {
        *level_selection = LevelSelection::Index(1);
    } else if input.just_pressed(KeyCode::Key3) {
        *level_selection = LevelSelection::Index(2);
    } else if input.just_pressed(KeyCode::Key4) {
        *level_selection = LevelSelection::Index(3);
    } else if input.just_pressed(KeyCode::Key5) {
        *level_selection = LevelSelection::Index(4);
    } else if input.just_pressed(KeyCode::Key6) {
        *level_selection = LevelSelection::Index(5);
    } else if input.just_pressed(KeyCode::Key7) {
        *level_selection = LevelSelection::Index(6);
    } else if input.just_pressed(KeyCode::Key8) {
        *level_selection = LevelSelection::Index(7);
    } else if input.just_pressed(KeyCode::Key9) {
        *level_selection = LevelSelection::Index(8);
    } else if input.just_pressed(KeyCode::Key0) {
        *level_selection = LevelSelection::Index(9);
    }
}

fn setup_world(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(LdtkWorldBundle {
        ldtk_handle: asset_server.load("levels/World.ldtk"),
        ..Default::default()
    });
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Wall;

#[derive(Clone, Debug, Default, Bundle, LdtkIntCell)]
pub struct WallBundle {
    wall: Wall,
}

#[derive(Clone, Debug, Default, Bundle)]
pub struct SpikeBundle {
    pub collider: Collider,
    pub label: Labeled,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
    pub death: crate::player::TouchDeath,
}

impl LdtkIntCell for SpikeBundle {
    fn bundle_int_cell(_int_grid_cell: IntGridCell, _layer_instance: &LayerInstance) -> Self {
        let rotation_constraints = LockedAxes::ROTATION_LOCKED;

        SpikeBundle {
            collider: Collider::cuboid(6., 4.),
            label: Labeled {
                name: String::from("spikes"),
            },
            rotation_constraints,
            active_events: ActiveEvents::COLLISION_EVENTS,
            ..Default::default()
        }
    }
}

#[derive(Clone, Default, Bundle, LdtkEntity)]
pub struct WheatBundle {
    pub collider: Collider,
    pub label: Labeled,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub rotation_constraints: LockedAxes,
    #[sprite_bundle("sprites/wheat_grown.png")]
    pub sprite_bundle: SpriteBundle,
}

/// FRom bevy_ecs_ldtk platformer example
/// Spawns rapier collisions for the walls of a level
///
/// You could just insert a ColliderBundle in to the WallBundle,
/// but this spawns a different collider for EVERY wall tile.
/// This approach leads to bad performance.
///
/// Instead, by flagging the wall tiles and spawning the collisions later,
/// we can minimize the amount of colliding entities.
///
/// The algorithm used here is a nice compromise between simplicity, speed,
/// and a small number of rectangle colliders.
/// In basic terms, it will:
/// 1. consider where the walls are
/// 2. combine wall tiles into flat "plates" in each individual row
/// 3. combine the plates into rectangles across multiple rows wherever possible
/// 4. spawn colliders for each rectangle
pub fn spawn_wall_collision(
    mut commands: Commands,
    wall_query: Query<(&GridCoords, &Parent), Added<Wall>>,
    parent_query: Query<&Parent, Without<Wall>>,
    level_query: Query<(Entity, &Handle<LdtkLevel>)>,
    levels: Res<Assets<LdtkLevel>>,
) {
    /// Represents a wide wall that is 1 tile tall
    /// Used to spawn wall collisions
    #[derive(Clone, Eq, PartialEq, Debug, Default, Hash)]
    struct Plate {
        left: i32,
        right: i32,
    }

    /// A simple rectangle type representing a wall of any size
    struct Rect {
        left: i32,
        right: i32,
        top: i32,
        bottom: i32,
    }

    // Consider where the walls are
    // storing them as GridCoords in a HashSet for quick, easy lookup
    //
    // The key of this map will be the entity of the level the wall belongs to.
    // This has two consequences in the resulting collision entities:
    // 1. it forces the walls to be split along level boundaries
    // 2. it lets us easily add the collision entities as children of the appropriate level entity
    let mut level_to_wall_locations: HashMap<Entity, HashSet<GridCoords>> = HashMap::new();

    wall_query.for_each(|(&grid_coords, parent)| {
        // An intgrid tile's direct parent will be a layer entity, not the level entity
        // To get the level entity, you need the tile's grandparent.
        // This is where parent_query comes in.
        if let Ok(grandparent) = parent_query.get(parent.get()) {
            level_to_wall_locations
                .entry(grandparent.get())
                .or_default()
                .insert(grid_coords);
        }
    });

    if !wall_query.is_empty() {
        level_query.for_each(|(level_entity, level_handle)| {
            if let Some(level_walls) = level_to_wall_locations.get(&level_entity) {
                let level = levels
                    .get(level_handle)
                    .expect("Level should be loaded by this point");

                let LayerInstance {
                    c_wid: width,
                    c_hei: height,
                    grid_size,
                    ..
                } = level
                    .level
                    .layer_instances
                    .clone()
                    .expect("Level asset should have layers")[0];

                // combine wall tiles into flat "plates" in each individual row
                let mut plate_stack: Vec<Vec<Plate>> = Vec::new();

                for y in 0..height {
                    let mut row_plates: Vec<Plate> = Vec::new();
                    let mut plate_start = None;

                    // + 1 to the width so the algorithm "terminates" plates that touch the right edge
                    for x in 0..width + 1 {
                        match (plate_start, level_walls.contains(&GridCoords { x, y })) {
                            (Some(s), false) => {
                                row_plates.push(Plate {
                                    left: s,
                                    right: x - 1,
                                });
                                plate_start = None;
                            }
                            (None, true) => plate_start = Some(x),
                            _ => (),
                        }
                    }

                    plate_stack.push(row_plates);
                }

                // combine "plates" into rectangles across multiple rows
                let mut rect_builder: HashMap<Plate, Rect> = HashMap::new();
                let mut prev_row: Vec<Plate> = Vec::new();
                let mut wall_rects: Vec<Rect> = Vec::new();

                // an extra empty row so the algorithm "finishes" the rects that touch the top edge
                plate_stack.push(Vec::new());

                for (y, current_row) in plate_stack.into_iter().enumerate() {
                    for prev_plate in &prev_row {
                        if !current_row.contains(prev_plate) {
                            // remove the finished rect so that the same plate in the future starts a new rect
                            if let Some(rect) = rect_builder.remove(prev_plate) {
                                wall_rects.push(rect);
                            }
                        }
                    }
                    for plate in &current_row {
                        rect_builder
                            .entry(plate.clone())
                            .and_modify(|e| e.top += 1)
                            .or_insert(Rect {
                                bottom: y as i32,
                                top: y as i32,
                                left: plate.left,
                                right: plate.right,
                            });
                    }
                    prev_row = current_row;
                }

                commands.entity(level_entity).with_children(|level| {
                    // Spawn colliders for every rectangle..
                    // Making the collider a child of the level serves two purposes:
                    // 1. Adjusts the transforms to be relative to the level for free
                    // 2. the colliders will be despawned automatically when levels unload
                    for wall_rect in wall_rects {
                        level
                            .spawn_empty()
                            .insert(Collider::cuboid(
                                (wall_rect.right as f32 - wall_rect.left as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                                (wall_rect.top as f32 - wall_rect.bottom as f32 + 1.)
                                    * grid_size as f32
                                    / 2.,
                            ))
                            .insert(RigidBody::Fixed)
                            .insert(Friction::new(1.0))
                            .insert(Labeled {
                                name: String::from("wall"),
                            })
                            .insert(Transform::from_xyz(
                                (wall_rect.left + wall_rect.right + 1) as f32 * grid_size as f32
                                    / 2.,
                                (wall_rect.bottom + wall_rect.top + 1) as f32 * grid_size as f32
                                    / 2.,
                                0.,
                            ))
                            .insert(GlobalTransform::default());
                    }
                });
            }
        });
    }
}
