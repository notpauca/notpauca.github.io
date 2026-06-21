use std::collections::HashSet;
use cgmath::{InnerSpace, Rad, Vector3};
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

        //camera angle
        let rendering_struct = &mut app.rendering_struct.borrow_mut();
        if self.keys.contains("ArrowRight") {
            rendering_struct.camera.stats.yaw+=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowLeft") {
            rendering_struct.camera.stats.yaw-=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowUp") {
            rendering_struct.camera.stats.pitch+=Rad(0.001*dt as f32);
        }
        if self.keys.contains("ArrowDown") {
            rendering_struct.camera.stats.pitch-=Rad(0.001*dt as f32);
        }


        if self.keys.contains("Enter") {
            web_sys::console::info_1(&format!("{:?}", rendering_struct.camera.stats).into())
        }

        let (yaw_sin, yaw_cos) = rendering_struct.camera.stats.yaw.0.sin_cos();

        //https://sotrh.github.io/learn-wgpu/intermediate/tutorial12-camera/#the-camera-controller
        //too lazy to remember the right math, so just stole it.
        let forward_vector = Vector3::new(yaw_cos, 0.0, yaw_sin).normalize();
        let right_vector = Vector3::new(-yaw_sin, 0.0, yaw_cos).normalize();
        rendering_struct.camera.stats.position += forward_vector * forward * 0.01 * dt as f32;
        rendering_struct.camera.stats.position += right_vector * right * 0.01 * dt as f32;
        rendering_struct.camera.stats.position.y += up * 0.01 * dt as f32;
    }
}
