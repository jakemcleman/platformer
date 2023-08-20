use crate::{
    pickup::{check_for_pickups, PickupCollector, PickupEvent},
    sprite_anim::SpriteAnimator,
};
use bevy::{prelude::*, sprite::Anchor};
use bevy_rapier2d::prelude::*;

pub struct ActorPlugin;

#[derive(Component, Clone)]
pub struct Actor {
    pub move_speed: f32,
    pub drag: f32,
    pub accel: f32,
    pub deccel: f32,
    pub up_gravity: f32,
    pub down_gravity: f32,
    pub jump_speed: f32,
    pub jump_time: f32,
    pub can_jump: bool,
}

#[derive(Component, Clone)]
pub struct ActorInputs {
    pub move_input: f32,
    pub jump_input: bool,
    pub grab_input: bool,
}


#[derive(Component, Default, Clone)]
pub struct Carrier {
    pub carrying: Option<Entity>,
    pub carry_offset: Vec3,
}

#[derive(Component, Clone)]
pub struct Carried {
    pub held_by: Entity,
}

#[derive(Component, Default, Clone)]
pub struct Carryable {
}

#[derive(Component, Default, Clone)]
pub struct ActorStatus {
    pub grounded: bool,
    pub velocity: Vec2,
    pub facing_left: bool,
    pub air_timer: f32,
    pub left_wall: bool,
    pub right_wall: bool,
    pub event: Option<ActorEvent>,
    pub last_dt: f32,
    pub carried: bool,
    pub carrying: bool,
}

#[derive(Component, Default, Clone)]
pub struct ActorAnimationStates {
    pub idle_row: usize,
    pub run_row: usize,
    pub jump_row: usize,
    pub fall_row: usize,
    pub idle_carry_row: usize,
    pub run_carry_row: usize,
    pub push_row: usize,
    pub yell_row: usize,
}

#[derive(Component, Default, Clone)]
pub struct ActorAudio {
    pub jump: Handle<AudioSource>,
    pub land: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub death: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub unlocked: Handle<AudioSource>,
    pub victory: Handle<AudioSource>,
}

#[derive(Debug, Clone)]
pub enum SquashStretchState {
    Restore,
    Squash,
    Stretch,
}

#[derive(Component, Debug, Default, Clone)]
pub struct Squashy {
    pub base_scale: Vec2,
    pub restore_time: f32,
    pub squash_scale: Vec2,
    pub squash_time: f32,
    pub stretch_scale: Vec2,
    pub stretch_time: f32,
    pub state: Option<SquashStretchState>,
    pub state_time: f32,
    pub from_pos: Vec2,
}

#[derive(Clone, Event)]
pub struct StartCarryEvent {
    pub picked_up: Entity,
    pub carrier: Entity,
}

#[derive(Clone, Event)]
pub struct EndCarryEvent {
    pub picked_up: Entity,
    pub carrier: Entity,
}

#[derive(Clone, Event)]
pub enum ActorEvent {
    Launched,
    Landed,
    _Hit,
    Died,
    Win,
    Pickup,
    _Unlock,
}

impl Squashy {
    pub fn change_state(&mut self, next: Option<SquashStretchState>) {
        self.from_pos = self.get_current_state_end_pos();
        self.state = next;
        self.state_time = 0.;
    }

    fn get_current_state_max_time(&self) -> f32 {
        if let Some(state) = &self.state {
            match state {
                SquashStretchState::Restore => self.restore_time,
                SquashStretchState::Squash => self.squash_time,
                SquashStretchState::Stretch => self.stretch_time,
            }
        } else {
            f32::MAX
        }
    }

    fn get_current_state_end_pos(&self) -> Vec2 {
        if let Some(state) = self.state.clone() {
            match state {
                SquashStretchState::Restore => Vec2::ONE,
                SquashStretchState::Squash => self.squash_scale,
                SquashStretchState::Stretch => self.stretch_scale,
            }
        } else {
            Vec2::ONE
        }
    }
}

impl Plugin for ActorPlugin {
    fn build(&self, app: &mut App) {
        app
        .add_event::<StartCarryEvent>()
        .add_event::<EndCarryEvent>()
        .add_systems(Update,(
            actor_status.before(actor_movement),
            actor_movement.before(actor_event_clear),
            actor_animations.before(actor_event_clear),
            actor_event_clear,
            squash_states.before(actor_event_clear),
            squash_animation,
            actor_carry.after(actor_carry_startstop)
        ))
        .add_systems(Update,
            (actor_squash_events, actor_pickup_effects, actor_carry_startstop, actor_audio)
                .before(actor_event_clear)
                .after(actor_status)
                .after(actor_movement)
                .after(check_for_pickups)
        )
        ;
    }
}

impl Default for Actor {
    fn default() -> Self {
        Actor {
            move_speed: 120.,
            drag: 0.3,
            accel: 1000.,
            deccel: 2000.,
            up_gravity: 300.,
            down_gravity: 500.,
            jump_speed: 800.,
            jump_time: 0.2,
            can_jump: true,
        }
    }
}

impl Default for ActorInputs {
    fn default() -> Self {
        ActorInputs {
            move_input: 0.,
            jump_input: false,
            grab_input: false,
        }
    }
}

pub fn actor_status(
    time: Res<Time>,
    mut actor_query: Query<(
        Entity,
        &Transform,
        &mut ActorStatus,
        &KinematicCharacterControllerOutput,
        Option<&Carrier>,
        Option<&Carried>,
    )>,
    rapier_context: Res<RapierContext>,
) {
    for (entity, transform, mut actor_status, controller_output, carrier_opt, carried_opt) in &mut actor_query {
        let distance = 1.0;
        let shape = Collider::capsule_y(5.5, 2.5);
        let shape_pos = transform.translation.truncate();
        let filter = QueryFilter::new()
            .exclude_sensors()
            .exclude_collider(entity)
            ;
            
        let mut cast_grounded = false;
        if let Some((_, _)) =
            rapier_context.cast_shape(shape_pos, 0., Vec2::new(0., -distance), &shape, 1., filter)
        {
            cast_grounded = true;
        }
        
        if !actor_status.grounded && cast_grounded {
            actor_status.event = Some(ActorEvent::Landed);
        }

        actor_status.grounded = cast_grounded;
        actor_status.velocity = controller_output.effective_translation / actor_status.last_dt;

        if actor_status.grounded {
            actor_status.air_timer = 0.;
            actor_status.velocity.y = 0.;
        } else {
            actor_status.air_timer += time.delta_seconds();
        }

        if let Some((_, _)) =
            rapier_context.cast_shape(shape_pos, 0., Vec2::new(distance, 0.), &shape, 1., filter)
        {
            actor_status.right_wall = true;
        } else {
            actor_status.right_wall = false;
        }

        if let Some((_, _)) =
            rapier_context.cast_shape(shape_pos, 0., Vec2::new(-distance, 0.), &shape, 1., filter)
        {
            actor_status.left_wall = true;
        } else {
            actor_status.left_wall = false;
        }
        
        actor_status.carried = carried_opt.is_some();
        
        if let Some(carrier) = carrier_opt {
            actor_status.carrying = carrier.carrying.is_some();
        }
        else {
            actor_status.carrying = false;
        }
    }
}

pub fn actor_pickup_effects(
    mut soul_pickup_events: EventReader<PickupEvent>,
    mut actor_statuses: Query<&mut ActorStatus, With<PickupCollector>>,
) {
    for ev in soul_pickup_events.iter() {
        if let Ok(mut status) = actor_statuses.get_mut(ev.collector_entity) {
            status.event = Some(ActorEvent::Pickup);
        }
    }
}

pub fn actor_carry(
    mut carrier_query: Query<(&Transform, &Carrier), Without<Carried>>,
    mut carryable_query: Query<(&mut Transform, &Carried)>,
) {
    for (carrier_transform, carrier) in &mut carrier_query {
        if let Some(carried_entity) = carrier.carrying {
            if let Ok((mut carryable_transform, _carried)) = carryable_query.get_mut(carried_entity) {
                carryable_transform.translation = carrier_transform.translation + carrier.carry_offset;
            }
        }
    }
}

pub fn actor_carry_startstop(
    mut carrier_query: Query<(Entity, &ActorInputs, &GlobalTransform, &mut Carrier), Without<Carried>>,
    mut carryable_query: Query<(&Carryable, Option<&Actor>, Option<&ActorInputs>, Option<&mut ActorStatus>, Option<&mut Carried>)>,
    rapier_context: Res<RapierContext>,
    mut start_carry_event_writer: EventWriter<StartCarryEvent>,
    mut end_carry_event_writer: EventWriter<EndCarryEvent>,
    mut commands: Commands,
) {
    for (carrier_entity, actor_input, carrier_transform, mut carrier) in &mut carrier_query {
        if actor_input.grab_input {
            //println!("grabby");
            
            if let Some(carried_entity) = carrier.carrying {
                // Already carrying, drop it
                end_carry_event_writer.send(EndCarryEvent {
                    carrier: carrier_entity,
                    picked_up: carried_entity,
                });
                
                carrier.carrying = None;
                commands.entity(carried_entity).remove::<Carried>();
                //println!("Drop");
            }
            else {
                // Not carrying, try to pick up
                let shape = Collider::capsule_x(5.5, 3.5);
                let filter = QueryFilter::new();
                let shape_pos = carrier_transform.translation().truncate();
                
                rapier_context.intersections_with_shape(shape_pos, 0., &shape, filter, |entity| -> bool {
                    if entity == carrier_entity {
                        return true;
                    }
                    else if let Ok((_, _, _, _, carried)) = carryable_query.get_mut(entity) {
                        if carried.is_none() {
                            start_carry_event_writer.send(StartCarryEvent{
                                carrier: carrier_entity,
                                picked_up: entity,
                            });
                            
                            //println!("Grab");
                            
                            carrier.carrying = Some(entity);
                            commands.entity(entity).insert(Carried{ held_by: carrier_entity });
                            
                            return false;
                        }
                    }
                    true
                });  
            }
        }
    }
    
    for (_carryable, opt_actor, opt_actor_inputs, opt_actor_status, opt_carried) in &mut carryable_query {
        if let Some(carried) = opt_carried {
            if let Some(actor) = opt_actor {
                if let Some(actor_input) = opt_actor_inputs {
                    if actor_input.jump_input {
                        if let Ok((_, _, _, mut carrier)) = carrier_query.get_mut(carried.held_by) {
                            commands.entity(carrier.carrying.unwrap()).remove::<Carried>();
                            carrier.carrying = None;
                        }
                        
                        if let Some(mut status) = opt_actor_status {
                            status.velocity.y = actor.jump_speed;
                            if status.grounded {
                                status.event = Some(ActorEvent::Launched);
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn actor_movement(
    time: Res<Time>,
    mut actor_query: Query<(&Actor, &ActorInputs, &mut ActorStatus, &mut KinematicCharacterController, Option<&Carried>)>,
) {
    for (actor, actor_input, mut status, mut controller, opt_carry) in &mut actor_query {
        // Track facing based on input seperately
        if actor_input.move_input > 0.1 {
            status.facing_left = false;
        } else if actor_input.move_input < -0.1 {
            status.facing_left = true;
        }
        
        // Skip all movement logic if being carried
        // TODO, let jump still happen, should break carrying
        let mut opt_carried_by = None;
        if let Some(carried) = opt_carry {
            opt_carried_by = Some(carried.held_by);
        }
        if let Some(_carried_by) = opt_carried_by {
            
        }
        else {
            let dir_match = actor_input.move_input.signum() == status.velocity.x.signum();
            let accel = if dir_match { actor.accel } else { actor.deccel };
            status.velocity.x += actor_input.move_input * accel * time.delta_seconds();
            
            if actor_input.move_input.abs() < 0.1 {
                status.velocity.x *= 1.0 - actor.drag;
            }
    
            status.velocity.x = status.velocity.x.clamp(-actor.move_speed, actor.move_speed);
    
            if (status.velocity.x > 0. && status.right_wall)
                || (status.velocity.x < 0. && status.left_wall)
            {
                status.velocity.x = 0.;
            }
    
            let state_can_jump = status.grounded || status.air_timer < actor.jump_time;

            if actor.can_jump && state_can_jump && actor_input.jump_input {
                status.velocity.y = actor.jump_speed;
    
                if status.grounded {
                    status.event = Some(ActorEvent::Launched);
                }
            } else if !status.grounded {
                status.velocity.y -= if status.velocity.y > 0. {
                    actor.down_gravity
                } else {
                    actor.up_gravity
                } * time.delta_seconds();
            }
    
            controller.translation = Some(time.delta_seconds() * status.velocity);
        }
        status.last_dt = time.delta_seconds();
    }
}

fn actor_animations(
    mut actor_query: Query<(
        &ActorStatus,
        &ActorAnimationStates,
        &mut SpriteAnimator,
        &mut TextureAtlasSprite,
    )>,
) {
    for (status, anim_states, mut animator, mut sprite) in &mut actor_query {
        if status.carrying {
            if status.velocity.x.abs() > 20. {
                animator.set_row(anim_states.run_carry_row);
            } else {
                animator.set_row(anim_states.idle_carry_row);
            }
        }
        else if status.carried {
            animator.set_row(anim_states.idle_row);
        }
        else if status.grounded {
            if status.velocity.x.abs() > 20. {
                animator.set_row(anim_states.run_row);
            } else {
                animator.set_row(anim_states.idle_row);
            }
        } else {
            if status.velocity.y > -10. {
                animator.set_row(anim_states.jump_row);
            } else {
                animator.set_row(anim_states.fall_row);
            }
        }

        sprite.flip_x = status.facing_left;
    }
}

fn actor_audio(
    actor_query: Query<(&ActorStatus, &ActorAudio)>, 
    mut commands: Commands
) {
    for (status, actor_sounds) in &actor_query {
        if let Some(event) = &status.event {
            commands.spawn(AudioSourceBundle {
                settings: PlaybackSettings::DESPAWN,
                source: match event {
                    ActorEvent::Launched => actor_sounds.jump.clone(),
                    ActorEvent::Landed => actor_sounds.land.clone(),
                    ActorEvent::_Hit => actor_sounds.hit.clone(),
                    ActorEvent::Died => actor_sounds.death.clone(),
                    ActorEvent::Pickup => actor_sounds.pickup.clone(),
                    ActorEvent::_Unlock => actor_sounds.unlocked.clone(),
                    ActorEvent::Win => actor_sounds.victory.clone(),
                }
            });
        }
    }
}


fn actor_squash_events(mut actor_query: Query<(&ActorStatus, &mut Squashy)>) {
    for (status, mut squish) in actor_query.iter_mut() {
        if let Some(event) = &status.event {
            match event {
                ActorEvent::Launched => squish.change_state(Some(SquashStretchState::Stretch)),
                ActorEvent::Landed => squish.change_state(Some(SquashStretchState::Squash)),
                _ => (),
            };
        }
    }
}

fn squash_states(time: Res<Time>, mut squish_query: Query<&mut Squashy>) {
    for mut squish in squish_query.iter_mut() {
        if let Some(squish_state) = squish.state.clone() {
            squish.state_time += time.delta_seconds();

            if squish.state_time > squish.get_current_state_max_time() {
                match squish_state {
                    SquashStretchState::Restore => squish.change_state(None),
                    SquashStretchState::Squash => {
                        squish.change_state(Some(SquashStretchState::Restore))
                    }
                    SquashStretchState::Stretch => {
                        squish.change_state(Some(SquashStretchState::Restore))
                    }
                };
                squish.state_time = 0.;
            }
        }
    }
}

fn squash_animation(mut squish_query: Query<(&Squashy, &mut TextureAtlasSprite)>) {
    for (squish, mut sprite) in squish_query.iter_mut() {
        if squish.state.is_some() {
            let t = squish.state_time / squish.get_current_state_max_time();
            let scale = squish.from_pos.lerp(squish.get_current_state_end_pos(), t);
            sprite.custom_size = Some(Vec2::new(
                scale.x * squish.base_scale.x,
                scale.y * squish.base_scale.y,
            ));

            let y_offset = (scale.y - 1.) / 2.;
            sprite.anchor = Anchor::Custom(Vec2::new(0., -y_offset));
        } else {
            sprite.custom_size = None;
            sprite.anchor = Anchor::Center;
        }
    }
}

pub fn actor_event_clear(mut actor_query: Query<&mut ActorStatus>) {
    for mut status in &mut actor_query {
        status.event = None;
    }
}
