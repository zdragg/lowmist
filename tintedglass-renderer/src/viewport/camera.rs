use bevy::{
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused},
};

pub struct MinecraftCameraPlugin;

impl Plugin for MinecraftCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, camera.spawn())
            .add_systems(Update, control);
    }
}

fn camera() -> impl Scene {
    bsn! {
        Camera3d::default()
        Transform::IDENTITY
        MinecraftCamera::default()
        Children [
            DirectionalLight {
                illuminance: 5000.0
            },
            Transform::IDENTITY
        ]
    }
}

#[derive(Component, Clone)]
pub(crate) struct MinecraftCamera {
    yaw: f32,
    pitch: f32,
    // Blocks per second
    speed: f32,
}

impl Default for MinecraftCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 30.0,
        }
    }
}

const TURN_SENSITIVITY: f32 = 0.005;
const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 1000.0;
const WHEEL_SCROLL_SENSITIVITY: f32 = 0.12;
const TRACKPAD_SCROLL_SENSITIVITY: f32 = 0.004;

impl MinecraftCamera {
    pub(crate) fn set_rotation(&mut self, yaw: f32, pitch: f32) {
        self.yaw = yaw;
        self.pitch = pitch;
    }

    fn rotate(&mut self, delta: Vec2) {
        self.yaw -= delta.x * TURN_SENSITIVITY;
        self.pitch = (self.pitch - delta.y * TURN_SENSITIVITY).clamp(
            -std::f32::consts::FRAC_PI_2 + 0.01,
            std::f32::consts::FRAC_PI_2 - 0.01,
        );
    }

    fn apply_rotation(&self, transform: &mut Transform) {
        transform.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn accelerate(&mut self, scroll: f32) {
        self.speed = (self.speed * (1.0 + scroll * 0.1)).clamp(MIN_SPEED, MAX_SPEED)
    }

    fn move_toward(&self, direction: Vec3, time: f32, transform: &mut Transform) {
        transform.translation += Quat::from_rotation_y(self.yaw) * direction * self.speed * time;
    }
}

fn control(
    time: Res<Time>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut focus_reader: MessageReader<WindowFocused>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera: Single<(&mut Transform, &mut MinecraftCamera)>,
) {
    // Cursor grab
    let focused = mouse_buttons.just_pressed(MouseButton::Left);
    if focused {
        cursor.grab_mode = CursorGrabMode::Locked;
        cursor.visible = false;
    }
    let lost_focus = keys.just_pressed(KeyCode::Escape)
        || matches!(focus_reader.read().last(), Some(message) if !message.focused);
    if lost_focus {
        cursor.grab_mode = CursorGrabMode::None;
        cursor.visible = true;
    }

    // Do nothing if unfocused
    if cursor.grab_mode == CursorGrabMode::None {
        return;
    }
    // Otherwise:

    let (transform, camera) = &mut *camera;
    let delta = mouse_motion.delta;

    // Rotation
    camera.rotate(delta);
    camera.apply_rotation(transform);

    // Speed change
    let scroll = mouse_scroll.delta;
    if scroll != Vec2::ZERO {
        match mouse_scroll.unit {
            MouseScrollUnit::Line => camera.accelerate(scroll.y * WHEEL_SCROLL_SENSITIVITY),
            MouseScrollUnit::Pixel => camera.accelerate(scroll.y * TRACKPAD_SCROLL_SENSITIVITY),
        }
    }

    // Movement
    let forward = Vec3::NEG_Z;
    let right = Vec3::X;
    let up = Vec3::Y;
    let mut direction = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        direction += forward;
    }
    if keys.pressed(KeyCode::KeyS) {
        direction -= forward;
    }
    if keys.pressed(KeyCode::KeyD) {
        direction += right;
    }
    if keys.pressed(KeyCode::KeyA) {
        direction -= right;
    }
    if keys.pressed(KeyCode::Space) {
        direction += up;
    }
    if keys.pressed(KeyCode::ShiftLeft) {
        direction -= up;
    }
    camera.move_toward(direction.normalize_or_zero(), time.delta_secs(), transform);
}
