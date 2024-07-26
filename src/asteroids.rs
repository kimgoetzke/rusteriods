use std::f32::consts::PI;
use std::ops::Range;

use bevy::color::palettes::css::*;
use bevy::math::{Vec2, Vec3};
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::*;
use bevy_prototype_lyon::shapes::Polygon;
use bevy_rapier2d::dynamics::AdditionalMassProperties;
use bevy_rapier2d::geometry::Collider;
use bevy_rapier2d::prelude::{Ccd, GravityScale, RigidBody, Velocity};
use rand::random;

use crate::camera::PIXEL_PERFECT_LAYERS;
use crate::game_state::GameState;
use crate::game_world::WORLD_SIZE;
use crate::shared::*;
use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

const ASTEROID_SPAWN_EVENT_RANGE: Range<u16> = 2..4;
const MAX_SPEED: f32 = 50.0;
const MAX_ROTATIONAL_SPEED: f32 = 2.5;
const MARGIN: f32 = 25.;

pub struct AsteroidPlugin;

impl Plugin for AsteroidPlugin {
  fn build(&self, app: &mut App) {
    app
      .add_event::<AsteroidSpawnEvent>()
      .add_event::<ResetAsteroidEvent>()
      .add_systems(OnEnter(GameState::Starting), reset_asteroids_system)
      .add_systems(
        Update,
        (spawn_smaller_asteroids_event, reset_asteroid_event).run_if(in_state(GameState::Playing)),
      )
      .add_systems(FixedUpdate, asteroid_wraparound_system);
  }
}

#[derive(Event)]
pub(crate) struct AsteroidSpawnEvent {
  pub(crate) category: Category,
  pub(crate) origin: Vec3,
}

#[derive(Event)]
pub(crate) struct ResetAsteroidEvent {}

#[derive(Component, Clone, Debug)]
pub(crate) struct Asteroid {
  pub category: Category,
  size_range: Range<f32>,
  sides: f32,
  collider: Collider,
  additional_mass: f32,
  pub(crate) score: u16,
}

impl Asteroid {
  fn large() -> Self {
    Self {
      category: Category::L,
      size_range: 20.0..40.0,
      sides: random_f32_range(12.0, 19.0),
      collider: Collider::ball(20.0),
      additional_mass: 30.0,
      score: 5,
    }
  }

  fn medium() -> Self {
    Self {
      category: Category::M,
      size_range: 10.0..20.0,
      sides: random_f32_range(7.0, 14.0),
      collider: Collider::ball(10.0),
      additional_mass: 17.5,
      score: 6,
    }
  }

  fn small() -> Self {
    Self {
      category: Category::S,
      size_range: 5.0..10.0,
      sides: random_f32_range(5.0, 9.0),
      collider: Collider::ball(5.0),
      additional_mass: 8.0,
      score: 7,
    }
  }

  fn shape(&self) -> Polygon {
    let mut points = Vec::with_capacity(self.sides as usize);
    let step = 2.0 * PI / (self.sides);
    for i in 0..self.sides as usize {
      let angle = step * i as f32;
      let radius = random_f32_range(self.size_range.start, self.size_range.end);
      let x = radius * angle.cos();
      let y = radius * angle.sin();
      points.push(Vec2::new(x, y));
    }
    let shape = { Polygon { points, closed: true } };
    shape
  }
}

pub fn spawn_asteroid_wave(count: u16, mut commands: Commands) {
  for _ in 0..count {
    let category = Category::L;
    let x = (random::<f32>() * WINDOW_WIDTH) - WINDOW_WIDTH / 2.0;
    let y = (random::<f32>() * WINDOW_HEIGHT) - WINDOW_HEIGHT / 2.0;
    spawn_asteroid(&mut commands, category, x, y);
  }
}

fn spawn_smaller_asteroids_event(mut asteroid_event: EventReader<AsteroidSpawnEvent>, mut commands: Commands) {
  for event in asteroid_event.read() {
    let spawn_count = random_u16_range(ASTEROID_SPAWN_EVENT_RANGE.start, ASTEROID_SPAWN_EVENT_RANGE.end);
    for _ in 0..spawn_count {
      let x = event.origin.x + (random::<f32>() * 20.0);
      let y = event.origin.y + (random::<f32>() * 20.0);
      spawn_asteroid(&mut commands, event.category, x, y);
    }
  }
}

fn reset_asteroids_system(mut commands: Commands, asteroid_query: Query<Entity, With<Asteroid>>) {
  for entity in asteroid_query.iter() {
    commands.entity(entity).despawn();
  }
}

fn reset_asteroid_event(
  mut reset_events: EventReader<ResetAsteroidEvent>,
  commands: Commands,
  asteroid_query: Query<Entity, With<Asteroid>>,
) {
  for _ in reset_events.read() {
    reset_asteroids_system(commands, asteroid_query);
    return;
  }
}

// TODO: Improve collider to support shapes more accurately
fn spawn_asteroid(commands: &mut Commands, category: Category, x: f32, y: f32) {
  let asteroid = match category {
    Category::XL => Asteroid::large(),
    Category::L => Asteroid::large(),
    Category::M => Asteroid::medium(),
    Category::S => Asteroid::small(),
  };
  commands
    .spawn((
      ShapeBundle {
        path: GeometryBuilder::build_as(&asteroid.shape()),
        spatial: SpatialBundle {
          transform: Transform {
            translation: Vec3::new(x, y, 0.0),
            ..default()
          },
          ..default()
        },
        ..Default::default()
      },
      PIXEL_PERFECT_LAYERS,
      Stroke::new(WHITE, 1.0),
    ))
    .insert(RigidBody::Dynamic)
    .insert(asteroid.collider.clone())
    // .insert(ActiveEvents::COLLISION_EVENTS) // Only makes sense if we handle collisions based on the combination of both entities
    .insert(GravityScale(0.0))
    .insert(AdditionalMassProperties::Mass(asteroid.additional_mass.clone()))
    .insert(Velocity {
      linvel: Vec2::new(
        random_f32_range(-MAX_SPEED, MAX_SPEED),
        random_f32_range(-MAX_SPEED, MAX_SPEED),
      ),
      angvel: random_f32_range(-MAX_ROTATIONAL_SPEED, MAX_ROTATIONAL_SPEED),
    })
    .insert(Ccd::enabled())
    .insert(asteroid);
}

fn asteroid_wraparound_system(mut asteroids: Query<&mut Transform, (With<RigidBody>, With<Asteroid>)>) {
  let extents = Vec3::new(WORLD_SIZE / 2.0, WORLD_SIZE / 2.0, 0.0);
  for mut transform in asteroids.iter_mut() {
    if transform.translation.x > (extents.x + MARGIN) {
      transform.translation.x = -extents.x - MARGIN;
    } else if transform.translation.x < (-extents.x - MARGIN) {
      transform.translation.x = extents.x + MARGIN;
    }
    if transform.translation.y > (extents.y + MARGIN) {
      transform.translation.y = -extents.y - MARGIN;
    } else if transform.translation.y < (-extents.y - MARGIN) {
      transform.translation.y = extents.y + MARGIN;
    }
  }
}
