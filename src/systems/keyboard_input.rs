use std::collections::HashSet;
use cgmath::{InnerSpace, Rad, Vector3, num_traits::clamp};
use crate::PortfolioApp;

#[derive(Default)]
#[repr(transparent)]
pub struct KeyboardInputSystem {
    keys: HashSet<String>
}

impl KeyboardInputSystem {
    pub fn keyboard_down(&mut self, event: web_sys::KeyboardEvent) {
        if self.keys.insert(event.code()) {
            web_sys::console::log_1(&format!("pressed key: {}", event.code()).into());
        }
    }

    pub fn keyboard_up(&mut self, event: web_sys::KeyboardEvent) {
        if self.keys.remove(&event.code()) {
            web_sys::console::log_1(&format!("released key: {}", event.code()).into());
        }
    }

    pub fn update(&mut self, app: &PortfolioApp, dt: f64) {
        //TODO(unimportant): abstract the movement logic in the Camera struct, so that I can make a keybind hashtable
        let (mut right, mut up, mut forward) = (0.0, 0.0, 0.0);

        //camera eye
        if self.keys.contains("KeyD") {
            right += 1.0;
        }
        if self.keys.contains("KeyA") {
            right -= 1.0;
        }
        if self.keys.contains("KeyW") {
            forward += 1.0;
        }
        if self.keys.contains("KeyS") {
            forward -= 1.0;
        }
        if self.keys.contains("Space") {
            up += 1.0;
        }
        if self.keys.contains("ShiftLeft") || self.keys.contains("ShiftRight") {
            up -= 1.0;
        }

        let rendering_struct = &mut app.rendering_struct.borrow_mut();
        let mut yaw = rendering_struct.camera.stats.yaw;
        let mut pitch = rendering_struct.camera.stats.pitch;

        //angles
        if self.keys.contains("ArrowRight") {
            yaw += Rad(0.001 * dt as f32);
        }
        if self.keys.contains("ArrowLeft") {
            yaw -= Rad(0.001 * dt as f32);
        }
        if self.keys.contains("ArrowUp") {
            pitch += Rad(0.001 * dt as f32);
        }
        if self.keys.contains("ArrowDown") {
            pitch -= Rad(0.001 * dt as f32);
        }
        rendering_struct.camera.stats.yaw = yaw;
        rendering_struct.camera.stats.pitch = clamp(pitch, Rad(-crate::consts::SAFE_FRAC_PI_2), Rad(crate::consts::SAFE_FRAC_PI_2));

        //zooming
        if self.keys.contains("Equal") {
            rendering_struct.camera.stats.fov = clamp(rendering_struct.camera.stats.fov - (0.05 * dt as f32), 1.0, 179.0);

        }
        if self.keys.contains("Minus") {
            rendering_struct.camera.stats.fov = clamp(rendering_struct.camera.stats.fov + (0.05 * dt as f32), 1.0, 179.0);
        }

        if self.keys.contains("Enter") {
            web_sys::console::info_1(&format!("{:?}", rendering_struct.camera.stats).into());
        }

        let (yaw_sin, yaw_cos) = rendering_struct.camera.stats.yaw.0.sin_cos();
        let forward_vector = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right_vector = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        rendering_struct.camera.stats.position += forward_vector * forward * 0.01 * dt as f32;
        rendering_struct.camera.stats.position += right_vector * right * 0.01 * dt as f32;
        rendering_struct.camera.stats.position.y += up * 0.01 * dt as f32;
    }
}
