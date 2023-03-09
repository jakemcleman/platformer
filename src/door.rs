use bevy::prelude::*;
use bevy_ecs_ldtk::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{sprite_anim::SpriteAnimator, world::Labeled};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default, Component)]
pub struct Door {
    pub next_level: usize,
    pub required_souls: usize,
}

#[derive(Clone, Default, Bundle)]
pub struct DoorBundle {
    pub sprite_sheet_bundle: SpriteSheetBundle,
    pub sprite_animator: SpriteAnimator,
    pub collider: Collider,
    pub label: Labeled,
    pub sensor: Sensor,
    pub active_events: ActiveEvents,
    pub door: Door,
}

impl LdtkEntity for DoorBundle {
    fn bundle_entity(
        entity_instance: &EntityInstance,
        _layer_instance: &LayerInstance,
        _tileset: Option<&Handle<Image>>,
        _tileset_definition: Option<&TilesetDefinition>,
        asset_server: &AssetServer,
        texture_atlases: &mut Assets<TextureAtlas>,
    ) -> Self {
        let mut door = Door::default();

        for field in entity_instance.field_instances.iter() {
            match field.identifier.as_str() {
                "NextLevel" => {
                    if let FieldValue::Int(Some(value)) = field.value {
                        door.next_level = value as usize;
                    }
                }
                "SoulsNeeded" => {
                    if let FieldValue::Int(Some(value)) = field.value {
                        door.required_souls = value as usize;
                    }
                }
                unknown => println!("Unknown field \"{}\" on LDtk door object!", unknown),
            }
        }

        let texture_handle = asset_server.load("sprites/door_closed.png");
        let texture_atlas =
            TextureAtlas::from_grid(texture_handle, Vec2::new(16., 32.), 4, 1, None, None);
        let texture_atlas_handle = texture_atlases.add(texture_atlas);

        DoorBundle {
            sprite_sheet_bundle: SpriteSheetBundle {
                transform: Transform::from_translation(Vec3::new(0., 0., 0.5)),
                texture_atlas: texture_atlas_handle,
                ..Default::default()
            },
            sprite_animator: SpriteAnimator::new(0, 3, 4, 0.2, true),
            collider: Collider::cuboid(8., 16.),
            label: Labeled {
                name: String::from("door to ") + door.next_level.to_string().as_str(),
            },
            sensor: Sensor,
            active_events: ActiveEvents::COLLISION_EVENTS,
            door,
        }
    }
}
