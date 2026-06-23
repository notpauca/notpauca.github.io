use std::collections::HashSet;
use cgmath::{InnerSpace, Quaternion, Rad, Rotation, Rotation3, Vector3};
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
        let (mut right, mut up, mut forward) = (0.0, 0.0, 0.0);

        //TODO: now make keybinds for things like movement, let user change them somehow.
        //TODO: abstract the movement logic in the Camera struct, so that I can make a keybind hashtable
        //camera eye
        if self.keys.contains("KeyD") {
            right +=1.0;
        }
        if self.keys.contains("KeyA") {
            right -=1.0;
        }
        if self.keys.contains("KeyW") {
            forward +=1.0;
        }
        if self.keys.contains("KeyS") {
            forward -=1.0;
        }
        if self.keys.contains("Space") {
            up+=1.0;
        }
        if self.keys.contains("ShiftLeft") || self.keys.contains("ShiftRight") {
            up-=1.0;
        }

        let rendering_struct = &mut app.rendering_struct.borrow_mut();
        let mut yaw = 0.0;
        let mut pitch = 0.0;
        let mut roll = 0.0;
        if self.keys.contains("ArrowRight") {
            yaw-=0.001 * dt as f32;
        }
        if self.keys.contains("ArrowLeft") {
            yaw+=0.001 * dt as f32;
        }
        if self.keys.contains("ArrowUp") {
            pitch+=0.001 * dt as f32;
        }
        if self.keys.contains("ArrowDown") {
            pitch-=0.001 * dt as f32;
        }
        if self.keys.contains("KeyE") {
            roll-=0.001 * dt as f32;
        }
        if self.keys.contains("KeyQ") {
            roll+=0.001 * dt as f32;
        }

        if self.keys.contains("Enter") {
            web_sys::console::info_1(&format!("{:?}", rendering_struct.camera.stats).into());
        }

        let forward_vector = rendering_struct.camera.stats.rotation.rotate_vector(-Vector3::unit_z());
        let right_vector = rendering_struct.camera.stats.rotation.rotate_vector(Vector3::unit_x());
        let up_vector = rendering_struct.camera.stats.rotation.rotate_vector(Vector3::unit_y());

        let yaw_quaternion = Quaternion::from_axis_angle(up_vector, Rad(yaw));
        let pitch_quaternion = Quaternion::from_axis_angle(rendering_struct.camera.stats.rotation*Vector3::unit_x(), Rad(pitch));
        let roll_quaternion = Quaternion::from_axis_angle(rendering_struct.camera.stats.rotation*Vector3::unit_z(), Rad(roll));

        rendering_struct.camera.stats.rotation = (roll_quaternion*pitch_quaternion*yaw_quaternion*rendering_struct.camera.stats.rotation).normalize();

        let speed = 0.01;
        let forward_vector = forward_vector * forward * speed * dt as f32;
        let right_vector = right_vector * right * speed * dt as f32;
        let up_vector = up_vector * up * speed * dt as f32;

        rendering_struct.camera.stats.position += forward_vector;
        rendering_struct.camera.stats.position += right_vector;
        rendering_struct.camera.stats.position += up_vector;
    }
}
