use bevy::{
    anti_alias::smaa::Smaa,
    core_pipeline::prepass::DepthPrepass,
    input::mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    prelude::*,
    render::occlusion_culling::OcclusionCulling,
    window::{CursorGrabMode, CursorOptions, PrimaryWindow, WindowFocused},
};

use crate::plugins::schematic::SchematicSpawned;

pub struct MinecraftCameraPlugin;

impl Plugin for MinecraftCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, control);
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::IDENTITY,
        MinecraftCamera::default(),
        DepthPrepass,
        OcclusionCulling,
        Msaa::Off,
        Smaa::default(),
        children![
            DirectionalLight {
                illuminance: 5000.0,
                ..default()
            },
            Transform::IDENTITY
        ],
    ));
}

#[derive(Component, Clone)]
pub struct MinecraftCamera {
    pub position: Vec3,
    pub velocity: Vec3,
    // Maximum blocks per second
    pub max_speed: f32,
    pub yaw: f32,
    pub pitch: f32,
}

impl Default for MinecraftCamera {
    fn default() -> Self {
        Self {
            position: Vec3::ZERO,
            velocity: Vec3::ZERO,
            max_speed: 11.1,
            yaw: 0.0,
            pitch: 0.0,
        }
    }
}

const TICKS_PER_SECOND: f32 = 20.0;
/// The fraction of the current velocity that survives per tick.
/// Every tick, horizontal velocity decreases by 9% while vertical velocity decreases by 40%
const DRAG: Vec3 = Vec3::new(0.91, 0.6, 0.91);
// Vertical speed is 0.675x of horizontal speed.
const SPEED_RATIO: Vec3 = Vec3::new(1.0, 0.675, 1.0);

const TURN_SENSITIVITY: f32 = 0.005;
const MIN_SPEED: f32 = 0.5;
const MAX_SPEED: f32 = 1000.0;
const WHEEL_SCROLL_SENSITIVITY: f32 = 0.12;
const TRACKPAD_SCROLL_SENSITIVITY: f32 = 0.004;

impl MinecraftCamera {
    /// Places the camera at a reasonable position to view the schematic with its bounds
    fn frame(&mut self, min_corner: IVec3, max_corner: IVec3) {
        let center_pos = (min_corner + max_corner + 1).as_vec3() / 2.0;
        // The radius of the smallest sphere that encompasses the entire schematic
        let radius = (max_corner - min_corner + 1).as_vec3().length() / 2.0;

        // Move toward the world's Y axis for radius * 1.5
        let direction = Vec3::new(0.0 - center_pos.x, 0.2, 0.0 - center_pos.z).normalize();
        self.position = center_pos + direction * radius * 1.5;

        // After moving toward that direction, look back toward the center position
        let rotation = Quat::look_to_rh(-direction, Vec3::Y).inverse();
        let (yaw, pitch, _roll) = rotation.to_euler(EulerRot::YXZ);
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

    fn apply_to(&self, transform: &mut Transform) {
        transform.translation = self.position;
        transform.rotation = Quat::from_rotation_y(self.yaw) * Quat::from_rotation_x(self.pitch);
    }

    fn accelerate(&mut self, scroll: f32) {
        self.max_speed = (self.max_speed * (1.0 + scroll * 0.1)).clamp(MIN_SPEED, MAX_SPEED)
    }

    fn move_toward(&mut self, direction: Vec3, dt: f32) {
        // Normalized vector describing the direction we're moving toward
        let wishdir = Quat::from_rotation_y(self.yaw) * direction;

        let decay = -TICKS_PER_SECOND * DRAG.ln() * dt;
        let max_speed = SPEED_RATIO * self.max_speed;

        self.velocity += wishdir * (max_speed * decay);
        self.velocity *= (-decay).exp();

        self.position += self.velocity * dt;
    }
}

// Handles input + applies MinecraftCamera to Transform
fn control(
    time: Res<Time>,
    mut schematic_spawned: MessageReader<SchematicSpawned>,
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut focus_reader: MessageReader<WindowFocused>,
    mut cursor: Single<&mut CursorOptions, With<PrimaryWindow>>,
    mut camera: Single<(&mut Transform, &mut MinecraftCamera)>,
) {
    let (transform, camera) = &mut *camera;

    // If schematic just spawned, initialize camera based on provided bounds
    if let Some(schematic_spawned) = schematic_spawned.read().last() {
        camera.frame(schematic_spawned.min_corner, schematic_spawned.max_corner);
    }

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

    let mut direction = Vec3::ZERO;

    // Handle input only if locked:
    if cursor.grab_mode == CursorGrabMode::Locked {
        // Rotation
        let mouse_delta = mouse_motion.delta;
        camera.rotate(mouse_delta);

        // Speed change
        let scroll_delta = mouse_scroll.delta;
        if scroll_delta != Vec2::ZERO {
            match mouse_scroll.unit {
                MouseScrollUnit::Line => {
                    camera.accelerate(scroll_delta.y * WHEEL_SCROLL_SENSITIVITY)
                }
                MouseScrollUnit::Pixel => {
                    camera.accelerate(scroll_delta.y * TRACKPAD_SCROLL_SENSITIVITY)
                }
            }
        }

        // Movement
        let forward = Vec3::NEG_Z;
        let right = Vec3::X;
        let up = Vec3::Y;
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
    }

    // Run even if direction is zero
    camera.move_toward(direction.normalize_or_zero(), time.delta_secs());

    // Apply camera to transform
    camera.apply_to(transform);
}
