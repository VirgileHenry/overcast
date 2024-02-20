use std::f32::consts::PI;

use bevy::{
    ecs::{
        component::Component,
        event::EventReader,
        query::With,
        system::{Query, Res},
    },
    input::{keyboard::KeyCode, mouse::MouseMotion, Input},
    math::{EulerRot, Quat, Vec2, Vec3},
    prelude::{default, Camera3dBundle},
    render::camera::Camera,
    time::Time,
    transform::components::Transform,
};

const SENSITIVITY_MULTIPLIER: f32 = 0.001;

pub fn create_camera() -> Camera3dBundle {
    Camera3dBundle {
        transform: Transform::from_xyz(100.0, 100.0, 150.0)
            .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
        ..default()
    }
}

#[derive(Component)]
pub struct CameraController {
    pub enabled: bool,
    pub sensitivity: f32,
    pub max_velocity: f32,
    pub velocity: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub roll: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        CameraController {
            enabled: true,
            sensitivity: 4.0,
            max_velocity: 100.0,
            velocity: Vec3::default(),
            yaw: 0.0,
            pitch: 0.0,
            roll: 0.0,
        }
    }
}

pub fn camera_animation_control(
    time: Res<Time>,
    mut mouse_motion: EventReader<MouseMotion>,
    keyboard_input: Res<Input<KeyCode>>,
    mut camera_query: Query<(&mut Transform, &mut CameraController), With<Camera>>,
) {
    if let Ok((mut transform, mut camera_controller)) = camera_query.get_single_mut() {
        if !camera_controller.enabled {
            mouse_motion.clear();
            return;
        }

        if keyboard_input.just_pressed(KeyCode::Escape) {
            camera_controller.enabled = !camera_controller.enabled;
        }

        let mut translate_input = Vec3::ZERO;
        let delta = time.delta_seconds();
        if keyboard_input.pressed(KeyCode::Z) {
            translate_input.z += 1.0;
        }
        if keyboard_input.pressed(KeyCode::S) {
            translate_input.z -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::Q) {
            translate_input.x -= 1.0;
        }
        if keyboard_input.pressed(KeyCode::D) {
            translate_input.x += 1.0;
        }
        if keyboard_input.pressed(KeyCode::Space) {
            translate_input.y += 1.0;
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) {
            translate_input.y -= 1.0;
        }

        if translate_input != Vec3::ZERO {
            let new_velocity = translate_input.normalize() * camera_controller.max_velocity;
            camera_controller.velocity = new_velocity;
        } else {
            camera_controller.velocity = Vec3::ZERO;
        }

        let right = transform.right();
        let up = transform.up();
        let forward = transform.forward();
        transform.translation += camera_controller.velocity.x * delta * right
            + camera_controller.velocity.y * delta * up
            + camera_controller.velocity.z * delta * forward;

        let mouse_move: Vec2 = mouse_motion.read().map(|delta_move| delta_move.delta).sum();
        camera_controller.yaw -=
            mouse_move.x * camera_controller.sensitivity * SENSITIVITY_MULTIPLIER;
        camera_controller.pitch = (camera_controller.pitch
            - mouse_move.y * camera_controller.sensitivity * SENSITIVITY_MULTIPLIER)
            .clamp(-PI / 2.0, PI / 2.0);
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            camera_controller.yaw,
            camera_controller.pitch,
        );
    }
}
