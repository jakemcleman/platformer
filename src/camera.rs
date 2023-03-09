use crate::player::Player;
use bevy::prelude::*;
use bevy_ecs_ldtk::LdtkLevel;

pub struct CameraPlugin;

#[derive(Resource, Default, Debug)]
pub struct WindowInfo {
    aspect_ratio: f32,
}

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::BLACK))
            .insert_resource(WindowInfo {
                aspect_ratio: 16. / 9.,
            })
            .add_systems(Startup, spawn_camera)
            .add_systems(Update, (camera_fit_inside_current_level, update_aspect_ratio))
            ;
    }
}

fn spawn_camera(mut commands: Commands) {
    //commands.spawn(PixelCameraBundle::from_resolution(VIEW_WIDTH, VIEW_HEIGHT));
    let camera = Camera2dBundle::default();
    commands.spawn(camera);
}

fn update_aspect_ratio(q_windows: Query<&Window>, mut window_info: ResMut<WindowInfo>) {
    let Ok(window) = q_windows.get_single() else {
        return;
    };

    window_info.aspect_ratio = window.width() / window.height();
}

pub fn camera_fit_inside_current_level(
    mut camera_query: Query<
        (
            &mut bevy::render::camera::OrthographicProjection,
            &mut Transform,
        ),
        Without<Player>,
    >,
    player_query: Query<&Transform, With<Player>>,
    level_query: Query<
        (&Transform, &Handle<LdtkLevel>),
        (Without<OrthographicProjection>, Without<Player>),
    >,
    ldtk_levels: Res<Assets<LdtkLevel>>,
    window_info: Res<WindowInfo>,
) {
    let mut player_center = Vec3::ZERO;
    let mut player_count = 0;
    
    for Transform {
        translation: player_translation,
        .. }
        in player_query.iter() {
        player_center += *player_translation;
        player_count += 1;
    }
    
    player_center /= player_count as f32;
        
    let player_translation = player_center;
    let (mut orthographic_projection, mut camera_transform) = camera_query.single_mut();
    for (level_transform, level_handle) in &level_query {
        if let Some(ldtk_level) = ldtk_levels.get(level_handle) {
            orthographic_projection.viewport_origin = Vec2::ZERO;
            
            let level = &ldtk_level.level;
            let level_ratio = level.px_wid as f32 / ldtk_level.level.px_hei as f32;
            
            if level_ratio > window_info.aspect_ratio {
                // level is wider than the screen
                let height = (level.px_hei as f32 / 9.).round() * 9.;
                let width = height * window_info.aspect_ratio;
                
                orthographic_projection.scaling_mode =
                            bevy::render::camera::ScalingMode::Fixed { width, height };
                            
                camera_transform.translation.x =
                            (player_translation.x - level_transform.translation.x - width / 2.)
                                .clamp(0., level.px_wid as f32 - width);
                camera_transform.translation.y = 0.;
            } else {
                // level is taller than the screen
                let width = (level.px_wid as f32 / 16.).round() * 16.;
                let height = width / window_info.aspect_ratio;
                
                orthographic_projection.scaling_mode =
                    bevy::render::camera::ScalingMode::Fixed { width, height };
                    
                camera_transform.translation.y =
                    (player_translation.y - level_transform.translation.y - height / 2.)
                        .clamp(0., level.px_hei as f32 - height);
                camera_transform.translation.x = 0.;
            }

            camera_transform.translation.x += level_transform.translation.x;
            camera_transform.translation.y += level_transform.translation.y;
        }
    }
}
