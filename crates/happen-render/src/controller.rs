use happen_core::{Input, KeyCode, Time, World};
use happen_math::Vec3;

#[derive(Clone, Debug)]
pub struct FpsController {
    pub yaw: f32,
    pub pitch: f32,
    pub speed: f32,
    pub sprint_multiplier: f32,
    pub sensitivity: f32,
    pub velocity_y: f32,
    pub jump_force: f32,
    pub gravity: f32,
    pub eye_height: f32,
    pub grounded: bool,
}

impl Default for FpsController {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            speed: 8.0,
            sprint_multiplier: 2.0,
            sensitivity: 0.003,
            velocity_y: 0.0,
            jump_force: 7.0,
            gravity: 20.0,
            eye_height: 1.7,
            grounded: true,
        }
    }
}

pub fn fps_controller_system(world: &mut World) {
    let (mouse_dx, mouse_dy, move_fwd, move_right, jump, sprint) = {
        let Some(input) = world.get_resource::<Input>() else {
            return;
        };
        if !input.cursor_locked {
            return;
        }
        let fwd = (input.key_pressed(KeyCode::W) as i32
            - input.key_pressed(KeyCode::S) as i32) as f32;
        let right = (input.key_pressed(KeyCode::D) as i32
            - input.key_pressed(KeyCode::A) as i32) as f32;
        let mdx = input.mouse_delta.0;
        let mdy = input.mouse_delta.1;
        (
            mdx,
            mdy,
            fwd,
            right,
            input.key_just_pressed(KeyCode::Space),
            input.key_pressed(KeyCode::LShift),
        )
    };

    let dt = world
        .get_resource::<Time>()
        .map(|t| t.delta_f32)
        .unwrap_or(1.0 / 60.0)
        .min(0.05);

    let controller_entity = {
        let mut found = None;
        for (entity, _) in world.query::<FpsController>() {
            found = Some(entity);
            break;
        }
        found
    };

    let Some(entity) = controller_entity else {
        return;
    };

    let ctrl = match world.get_component::<FpsController>(entity) {
        Some(c) => c.clone(),
        None => return,
    };

    let pos = match world.get_component::<happen_math::Transform>(entity) {
        Some(t) => t.position,
        None => return,
    };

    let mut yaw = ctrl.yaw - mouse_dx * ctrl.sensitivity;
    let pitch = (ctrl.pitch - mouse_dy * ctrl.sensitivity)
        .clamp(-89.0_f32.to_radians(), 89.0_f32.to_radians());

    if yaw > std::f32::consts::TAU {
        yaw -= std::f32::consts::TAU;
    } else if yaw < -std::f32::consts::TAU {
        yaw += std::f32::consts::TAU;
    }

    let actual_speed = if sprint {
        ctrl.speed * ctrl.sprint_multiplier
    } else {
        ctrl.speed
    };

    let forward_xz = Vec3::new(-yaw.sin(), 0.0, -yaw.cos());
    let right_dir = Vec3::new(yaw.cos(), 0.0, -yaw.sin());

    let mut new_pos = pos;
    let move_dir = forward_xz * move_fwd + right_dir * move_right;
    if move_dir.length_squared() > 0.0 {
        new_pos += move_dir.normalize() * actual_speed * dt;
    }

    let mut vel_y = ctrl.velocity_y;
    vel_y -= ctrl.gravity * dt;
    if jump && ctrl.grounded {
        vel_y = ctrl.jump_force;
    }
    new_pos.y += vel_y * dt;

    let grounded = new_pos.y <= ctrl.eye_height;
    if grounded {
        new_pos.y = ctrl.eye_height;
        vel_y = 0.0;
    }

    let look_dir = Vec3::new(
        -yaw.sin() * pitch.cos(),
        pitch.sin(),
        -yaw.cos() * pitch.cos(),
    );
    let target = new_pos + look_dir;

    if let Some(c) = world.get_component_mut::<FpsController>(entity) {
        c.yaw = yaw;
        c.pitch = pitch;
        c.velocity_y = vel_y;
        c.grounded = grounded;
    }

    if let Some(t) = world.get_component_mut::<happen_math::Transform>(entity) {
        t.position = new_pos;
    }

    if let Some(cam) = world.get_component_mut::<crate::Camera>(entity) {
        cam.target = Some(target);
    }
}
