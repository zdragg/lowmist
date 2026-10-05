use bevy::{
    input::{
        gestures::PinchGesture,
        mouse::{AccumulatedMouseMotion, AccumulatedMouseScroll, MouseScrollUnit},
    },
    prelude::*,
};

pub struct OrbitCameraPlugin;

impl Plugin for OrbitCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, control);
    }
}

#[derive(Component)]
pub struct OrbitCamera {
    /// Position of the focused point
    pub focus: Vec3,
    /// Distance from focused point
    pub distance: f32,
    /// Horizontal angle, around the y axis, of the direction the camera is facing (which is toward the focus point)
    pub yaw: f32,
    /// Vertical angle, around the x axis, of the direction the camera is facing (which is toward the focus point)
    pub pitch: f32,
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            focus: Vec3::ZERO,
            distance: 10.0,
            yaw: -0.7,
            pitch: 0.5,
        }
    }
}

/// How fast the camera rotates around the scene
const ORBIT_SENSITIVITY: f32 = 0.005;
/// How fast the camera slides in front of the scene
const PAN_SENSITIVITY: f32 = 0.0015;
/// Minimum distance from focused point
const MIN_DISTANCE: f32 = 0.5;
/// Maximum distance from focused point
const MAX_DISTANCE: f32 = 500.0;

const WHEEL_ZOOM_SENSITIVITY: f32 = 0.12;
const TRACKPAD_ZOOM_SENSITIVITY: f32 = 0.01;
const PINCH_ZOOM_SENSITIVITY: f32 = 3.0;
const TRACKPAD_MOTION_SCALE: f32 = 0.4;

impl OrbitCamera {
    /// Change yaw and pitch based on how far the pointer dragged (in pixels).
    fn orbit_by(&mut self, delta: Vec2) {
        self.yaw -= delta.x * ORBIT_SENSITIVITY;
        self.pitch -= delta.y * ORBIT_SENSITIVITY;
    }

    /// Slide the point that the camera is focused on, so it looks like as if
    /// the entire scene is moving.
    fn pan(&mut self, transform: &Transform, delta: Vec2) {
        let (right, up) = (transform.right(), transform.up());
        let scale = PAN_SENSITIVITY * self.distance; // The further away, the faster it slides
        self.focus += (-right * delta.x + up * delta.y) * scale;
    }

    /// Move the camera nearer or further. Positive `amount` zooms in.
    fn zoom(&mut self, amount: f32) {
        self.distance = (self.distance * (1.0 - amount)).clamp(MIN_DISTANCE, MAX_DISTANCE);
    }

    /// Calculate actual camera rotation + translation from the struct, then apply.
    fn apply_to(&self, transform: &mut Transform) {
        let rot =
            Quat::from_axis_angle(Vec3::Y, self.yaw) * Quat::from_axis_angle(Vec3::X, self.pitch);
        transform.rotation = rot;
        // +Z points out of the screen, so this Vec3 is "backwards" toward the camera.
        // Multiply by this by rotation to get opposite of the current rotation,
        // scaled by self.distance. Add that to the position of the focused block to get
        // camera position.
        //
        // I don't really get why this works, but it works.
        transform.translation = self.focus + rot * Vec3::new(0.0, 0.0, self.distance);
    }
}

/// Spawn a camera in its default position.
fn spawn_camera(mut commands: Commands) {
    let orbit = OrbitCamera::default();
    let mut transform = Transform::IDENTITY;
    orbit.apply_to(&mut transform);

    commands.spawn((Camera3d::default(), transform, orbit));
}

/// Handles input.
fn control(
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse_motion: Res<AccumulatedMouseMotion>,
    mouse_scroll: Res<AccumulatedMouseScroll>,
    mut pinch_reader: MessageReader<PinchGesture>,
    mut camera: Single<(&mut Transform, &mut OrbitCamera)>,
) {
    let (transform, orbit) = &mut *camera;

    let shift = keys.any_pressed([KeyCode::ShiftLeft, KeyCode::ShiftRight]);
    let ctrl = keys.any_pressed([
        KeyCode::ControlLeft,
        KeyCode::ControlRight,
        KeyCode::SuperLeft,
        KeyCode::SuperRight,
    ]);

    if mouse_buttons.pressed(MouseButton::Middle) {
        let delta = mouse_motion.delta;
        if delta != Vec2::ZERO {
            if shift {
                orbit.pan(transform, delta);
            } else {
                orbit.orbit_by(delta);
            }
        }
    }

    let scroll = mouse_scroll.delta;
    if scroll != Vec2::ZERO {
        match mouse_scroll.unit {
            MouseScrollUnit::Line => orbit.zoom(scroll.y * WHEEL_ZOOM_SENSITIVITY),
            MouseScrollUnit::Pixel => {
                let delta = scroll * TRACKPAD_MOTION_SCALE;
                if ctrl {
                    orbit.zoom(delta.y * TRACKPAD_ZOOM_SENSITIVITY);
                } else if shift {
                    orbit.pan(transform, delta);
                } else {
                    orbit.orbit_by(delta);
                }
            }
        }
    }

    let pinch: f32 = pinch_reader.read().map(|g| g.0).sum();
    if pinch != 0.0 {
        orbit.zoom(pinch * PINCH_ZOOM_SENSITIVITY);
    }

    orbit.apply_to(transform);
}
