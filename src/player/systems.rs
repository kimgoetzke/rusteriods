use crate::{BOUNDS, SHOOTING_COOLDOWN};
use bevy::prelude::*;

use super::components::Player;

pub fn spawn_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let player_handle = asset_server.load("player_base.png");
    commands
        .spawn(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..default()
            },
            texture: player_handle,
            ..default()
        })
        .insert(Player {
            movement_speed: 500.0,
            rotation_speed: f32::to_radians(360.0),
            velocity: Default::default(),
            shooting_cooldown: SHOOTING_COOLDOWN,
        });
}

pub(crate) fn player_movement_system(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Player, &mut Transform)>,
) {
    for (mut player, mut transform) in query.iter_mut() {
        // Update rotation
        let rotation_factor = if keyboard_input.pressed(KeyCode::KeyA) {
            1.0
        } else if keyboard_input.pressed(KeyCode::KeyD) {
            -1.0
        } else {
            0.0
        };
        transform.rotate_z(rotation_factor * player.rotation_speed * time.delta_seconds());

        // Set acceleration
        if keyboard_input.pressed(KeyCode::KeyW) {
            let acceleration = transform.rotation * Vec3::Y * player.movement_speed;
            player.velocity += acceleration * time.delta_seconds();
        }

        // Apply friction
        player.velocity *= 0.995;

        // Update player position
        transform.translation += player.velocity * time.delta_seconds();

        // Wrap around the screen
        let extents = Vec3::from((BOUNDS / 2.0, 0.0));
        if transform.translation.x > extents.x {
            transform.translation.x = -extents.x;
        } else if transform.translation.x < -extents.x {
            transform.translation.x = extents.x;
        }
        if transform.translation.y > extents.y {
            transform.translation.y = -extents.y;
        } else if transform.translation.y < -extents.y {
            transform.translation.y = extents.y;
        }

        // Update shooting cooldown
        if player.shooting_cooldown > 0.0 {
            player.shooting_cooldown -= time.delta_seconds();
        }
    }
}