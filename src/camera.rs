use crate::resources::RayMeshHits;
use bevy::input::mouse::{MouseButton, MouseMotion, MouseScrollUnit, MouseWheel};
use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use bevy::window::PrimaryWindow;
use std::f32::consts::FRAC_PI_2;

#[derive(Bundle, Default)]
pub struct PanOrbitCameraBundle {
    pub camera: Camera3dBundle,
    pub state: FirstPersonState,
    pub settings: FirstPersonSettings,
}

#[derive(Component)]
pub struct FirstPersonState {
    pub position: Vec3,
    pub pitch: f32,
    pub yaw: f32,
}

impl Default for FirstPersonState {
    fn default() -> Self {
        FirstPersonState {
            position: Vec3::ZERO,
            pitch: 0.0,
            yaw: 0.0,
        }
    }
}

#[derive(Component)]
pub struct FirstPersonSettings {
    pub sensitivity: f32,
    pub move_front_key: InputSource,
    pub move_back_key: InputSource,
    pub move_right_key: InputSource,
    pub move_left_key: InputSource,
    pub move_up_key: InputSource,
    pub move_down_key: InputSource,
    pub carve_key: InputSource,
    pub toggle_grab_key: InputSource,
    pub movement_speed: f32,
}

impl Default for FirstPersonSettings {
    fn default() -> Self {
        FirstPersonSettings {
            sensitivity: 0.002,
            move_front_key: InputSource::Key(KeyCode::KeyW),
            move_back_key: InputSource::Key(KeyCode::KeyS),
            move_left_key: InputSource::Key(KeyCode::KeyA),
            move_right_key: InputSource::Key(KeyCode::KeyD),
            move_up_key: InputSource::Key(KeyCode::KeyR),
            move_down_key: InputSource::Key(KeyCode::KeyZ),
            carve_key: InputSource::Key(KeyCode::KeyC),
            toggle_grab_key: InputSource::Key(KeyCode::Escape),
            movement_speed: 0.2, // Speed for WASD movement
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputSource {
    Key(KeyCode),
    Mouse(MouseButton),
}

pub fn spawn_camera(mut commands: Commands) {
    let mut camera = PanOrbitCameraBundle::default();
    camera.state.position = Vec3::new(0.0, 0.5, 0.0); // Center the camera on the cube
    camera.state.pitch = 30.0f32.to_radians();
    camera.state.yaw = 45.0f32.to_radians();

    commands.spawn(camera);
}

pub fn handle_camera(
    kbd: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    mut evr_motion: EventReader<MouseMotion>,
    mut evr_scroll: EventReader<MouseWheel>,
    mut q_camera: Query<(&FirstPersonSettings, &mut FirstPersonState, &mut Transform)>,
    mut raycast: MeshRayCast,
    mut ray_hits: ResMut<RayMeshHits>,
) {
    let mut total_motion: Vec2 = evr_motion.read().map(|ev| ev.delta).sum();
    total_motion.y = -total_motion.y;

    let mut total_scroll_lines = Vec2::ZERO;
    let mut total_scroll_pixels = Vec2::ZERO;
    for ev in evr_scroll.read() {
        match ev.unit {
            MouseScrollUnit::Line => {
                total_scroll_lines.x += ev.x;
                total_scroll_lines.y -= ev.y;
            }
            MouseScrollUnit::Pixel => {
                total_scroll_pixels.x += ev.x;
                total_scroll_pixels.y -= ev.y;
            }
        }
    }

    for (settings, mut state, mut transform) in &mut q_camera {
        if is_pressed(&settings.carve_key, &kbd, &mouse_button_input) {
            let ray = Ray3d::new(transform.translation, transform.forward());
            let hits = raycast.cast_ray(ray, &RayCastSettings::default());

            if let Some(closest_hit) = hits.first() {
                ray_hits.as_mut().0.push_back(closest_hit.clone());
            }
        }

        state.yaw -= total_motion.x * settings.sensitivity;
        state.pitch += total_motion.y * settings.sensitivity;
        // Clamp pitch to avoid flipping
        state.pitch = state.pitch.clamp(-FRAC_PI_2, FRAC_PI_2);

        // Handle WASD movement
        let mut movement = Vec3::ZERO;
        if is_pressed(&settings.move_front_key, &kbd, &mouse_button_input) {
            movement += Vec3::from(transform.forward());
        }
        if is_pressed(&settings.move_back_key, &kbd, &mouse_button_input) {
            movement -= Vec3::from(transform.forward());
        }
        if is_pressed(&settings.move_left_key, &kbd, &mouse_button_input) {
            movement -= Vec3::from(transform.right());
        }
        if is_pressed(&settings.move_right_key, &kbd, &mouse_button_input) {
            movement += Vec3::from(transform.right());
        }
        if is_pressed(&settings.move_up_key, &kbd, &mouse_button_input) {
            movement += Vec3::from(transform.up());
        }
        if is_pressed(&settings.move_down_key, &kbd, &mouse_button_input) {
            movement -= Vec3::from(transform.up());
        }

        if movement != Vec3::ZERO {
            state.position += movement.normalize() * settings.movement_speed
        }

        transform.rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
        transform.translation = state.position;
    }
}

fn is_pressed(
    input_source: &InputSource,
    kbd: &Res<ButtonInput<KeyCode>>,
    mouse_button_input: &Res<ButtonInput<MouseButton>>,
) -> bool {
    match input_source {
        InputSource::Key(key) => kbd.pressed(*key),
        InputSource::Mouse(button) => mouse_button_input.pressed(*button),
    }
}

fn is_just_pressed(
    input_source: &InputSource,
    kbd: &Res<ButtonInput<KeyCode>>,
    mouse_button_input: &Res<ButtonInput<MouseButton>>,
) -> bool {
    match input_source {
        InputSource::Key(key) => kbd.just_pressed(*key),
        InputSource::Mouse(button) => mouse_button_input.just_pressed(*button),
    }
}

// This system grabs the mouse when the left mouse button is pressed
// and releases it when the escape key is pressed
pub fn grab_mouse(
    mut window: Single<&mut Window>,
    kbd: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    q_camera_settings: Query<&FirstPersonSettings>,
) {
    let settings = q_camera_settings.get_single().unwrap();
    if is_just_pressed(&settings.toggle_grab_key, &kbd, &mouse_button_input) {
        window.cursor_options.visible = !window.cursor_options.visible;
        window.cursor_options.grab_mode = match window.cursor_options.grab_mode {
            CursorGrabMode::Locked => CursorGrabMode::None,
            _ => CursorGrabMode::Locked,
        };
    }
}

// #[cfg(target_os = "windows")]
// pub fn cursor_recenter(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
//     let mut primary_window = q_windows.single_mut();
//     let center = Vec2::new(primary_window.width() / 2.0, primary_window.height() / 2.0);
//     println!("{:?}", center);
//     primary_window.set_cursor_position(Some(center));
// }
