use bevy::app::{App, Plugin};
use bevy::prelude::*;
use bevy_rapier2d::dynamics::{AdditionalMassProperties, Ccd, GravityScale, RigidBody, Velocity};
use bevy_rapier2d::geometry::{ActiveEvents, Collider};

use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::game_state::GameState;
use crate::player::Player;
use crate::projectile::{ProjectileInfo, ProjectileSpawnEvent};
use crate::shared::{random_f32_range, RED};

const SPEED: f32 = 50.;
const SHOOTING_COOLDOWN: f32 = 1.;

pub struct UfoPlugin;

impl Plugin for UfoPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_systems(FixedUpdate, ufo_movement_system.run_if(in_state(GameState::Playing)))
      .add_systems(Update, ufo_shooting_system.run_if(in_state(GameState::Playing)));
  }
}

#[derive(Component)]
pub struct Ufo {
  shooting_cooldown: f32,
}

pub fn spawn_ufo_wave(count: u16, mut commands: &mut Commands, asset_server: &Res<AssetServer>) {
  for _ in 0..count {
    spawn_ufo(&mut commands, &asset_server);
  }
}

fn spawn_ufo(commands: &mut &mut Commands, asset_server: &Res<AssetServer>) {
  let ufo_handle = asset_server.load("enemy_ufo.png");
  commands.spawn((
    SpriteBundle {
      texture: ufo_handle,
      ..default()
    },
    PIXEL_PERFECT_LAYERS,
    Name::new("UFO"),
    RigidBody::Dynamic,
    Collider::ball(2.),
    ActiveEvents::COLLISION_EVENTS,
    GravityScale(0.),
    Velocity {
      linvel: Vec2::new(0., 0.),
      angvel: 1.,
    },
    AdditionalMassProperties::Mass(4.),
    Ccd::enabled(),
    Ufo {
      shooting_cooldown: SHOOTING_COOLDOWN,
    },
  ));
}

fn ufo_movement_system(
  mut ufo_query: Query<(&Transform, &mut Velocity), With<Ufo>>,
  player_query: Query<(Entity, &Transform), With<Player>>,
) {
  for (transform, mut velocity) in ufo_query.iter_mut() {
    if let Ok(player) = player_query.get_single().as_ref() {
      let direction = player.1.translation - transform.translation;
      let direction = direction / direction.length();
      velocity.linvel = Vec2::new(direction.x * SPEED, direction.y * SPEED);
    }
  }
}

fn ufo_shooting_system(
  time: Res<Time>,
  mut query: Query<(&mut Ufo, &Transform)>,
  mut projective_spawn_event: EventWriter<ProjectileSpawnEvent>,
  player_query: Query<&Transform, With<Player>>,
) {
  for (mut ufo, transform) in query.iter_mut() {
    // Shoot a projectile if the cooldown is over
    if ufo.shooting_cooldown <= 0. {
      let origin_forward = get_origin_forward(&player_query, transform);
      let projectile_info = ProjectileInfo {
        speed: 100.,
        life_time: 0.,
        max_life_time: 1.5,
        cooldown: 1.,
        collider: Collider::cuboid(1.25, 1.25),
        sprite: Sprite {
          color: RED,
          custom_size: Some(Vec2::new(2.5, 2.5)),
          ..default()
        },
      };
      ufo.shooting_cooldown = projectile_info.cooldown;
      projective_spawn_event.send(ProjectileSpawnEvent {
        projectile_info,
        origin_transform: transform.clone(),
        origin_forward,
        spawn_position: transform.translation + origin_forward * 10.,
      });
    }

    // Update the shooting cooldown
    if ufo.shooting_cooldown > 0. {
      ufo.shooting_cooldown -= time.delta_seconds();
    }
  }
}

fn get_origin_forward(player_query: &Query<&Transform, With<Player>>, transform: &Transform) -> Vec3 {
  if let Ok(player) = player_query.get_single().as_ref() {
    let direction = player.translation - transform.translation;
    let direction = direction / direction.length();
    return direction;
  }
  let random_number = random_f32_range(-1., 1.);
  let anchor = if random_number > 0. { 1. } else { -1. };
  Vec3::new(random_number, anchor - random_number, 0.)
}