use std::time::Duration;
use std::{f32::consts::PI, str::FromStr};

use bevy::window::PresentMode;
use bevy::{
    animation::RepeatAnimation, pbr::CascadeShadowConfigBuilder, prelude::*, window::CursorGrabMode,
};
use camera::{camera_animation_control, create_camera};

use crate::camera::CameraController;
mod camera;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            (
                setup_scene_once_loaded,
                keyboard_animation_control,
                camera_animation_control,
            ),
        )
        .run();
}

#[derive(Resource)]
struct Animations(Vec<Handle<AnimationClip>>);

fn setup(
    windows: Query<&mut Window>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    window_setup(windows);

    // Insert a resource with the current scene information
    commands.insert_resource(Animations(vec![
        asset_server.load("models/animated/Fox.glb#Animation2"),
        asset_server.load("models/animated/Fox.glb#Animation1"),
        asset_server.load("models/animated/Fox.glb#Animation0"),
    ]));

    // Camera
    let mut camera = commands.spawn(create_camera());
    camera.insert(CameraController::default());

    // Plane
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Plane::from_size(500000.0).into()),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });

    // Light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(Quat::from_euler(EulerRot::ZYX, 0.0, 1.0, -PI / 4.)),
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            first_cascade_far_bound: 200.0,
            maximum_distance: 400.0,
            ..default()
        }
        .into(),
        ..default()
    });

    // Fox
    commands.spawn(SceneBundle {
        scene: asset_server.load("models/animated/Fox.glb#Scene0"),
        ..default()
    });

    println!("Animation controls:");
    println!("  - spacebar: play / pause");
    println!("  - arrow up / down: speed up / slow down animation playback");
    println!("  - arrow left / right: seek backward / forward");
    println!("  - digit 1 / 3 / 5: play the animation <digit> times");
    println!("  - L: loop the animation forever");
    println!("  - return: change animation");
}

fn window_setup(mut windows: Query<&mut Window>) {
    if let Ok(mut window) = windows.get_single_mut() {
        window.cursor.visible = false;
        window.cursor.grab_mode = CursorGrabMode::Confined;
        window.focused = true;
        window.prevent_default_event_handling = true; // Only for web, best to avoid side effects
        window.title = String::from_str("Overcast").unwrap();
        window.present_mode = PresentMode::AutoVsync;
    }
}

// Once the scene is loaded, start the animation
fn setup_scene_once_loaded(
    animations: Res<Animations>,
    mut players: Query<&mut AnimationPlayer, Added<AnimationPlayer>>,
) {
    for mut player in &mut players {
        player.play(animations.0[0].clone_weak()).repeat();
    }
}

fn keyboard_animation_control(
    keyboard_input: Res<Input<KeyCode>>,
    mut animation_players: Query<&mut AnimationPlayer>,
    animations: Res<Animations>,
    mut current_animation: Local<usize>,
) {
    for mut player in &mut animation_players {
        if keyboard_input.just_pressed(KeyCode::Space) {
            if player.is_paused() {
                player.resume();
            } else {
                player.pause();
            }
        }

        if keyboard_input.just_pressed(KeyCode::Up) {
            let speed = player.speed();
            player.set_speed(speed * 1.2);
        }

        if keyboard_input.just_pressed(KeyCode::Down) {
            let speed = player.speed();
            player.set_speed(speed * 0.8);
        }

        if keyboard_input.just_pressed(KeyCode::Left) {
            let elapsed = player.seek_time();
            player.seek_to(elapsed - 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Right) {
            let elapsed = player.seek_time();
            player.seek_to(elapsed + 0.1);
        }

        if keyboard_input.just_pressed(KeyCode::Return) {
            *current_animation = (*current_animation + 1) % animations.0.len();
            player
                .play_with_transition(
                    animations.0[*current_animation].clone_weak(),
                    Duration::from_millis(250),
                )
                .repeat();
        }

        if keyboard_input.just_pressed(KeyCode::Key1) {
            player.set_repeat(RepeatAnimation::Count(1));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::Key3) {
            player.set_repeat(RepeatAnimation::Count(3));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::Key5) {
            player.set_repeat(RepeatAnimation::Count(5));
            player.replay();
        }

        if keyboard_input.just_pressed(KeyCode::L) {
            player.set_repeat(RepeatAnimation::Forever);
        }
    }
}
