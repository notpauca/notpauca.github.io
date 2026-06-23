use cgmath::{Quaternion, Rad, Rotation, Rotation3, Vector3};
use crate::{consts, PortfolioApp};

#[derive(Default)]
pub struct MouseInputSystem {
    mouse_movement_delta: (f32, f32),
    mouse_locked: bool
}

impl MouseInputSystem {
    pub fn mouse_moved(&mut self, event: web_sys::PointerEvent) {
        if self.mouse_locked {
            self.mouse_movement_delta = (event.movement_x() as f32, event.movement_y() as f32);
        }
    }

    pub fn update(&mut self, app: &PortfolioApp) {
        let rendering_struct = &mut app.rendering_struct.borrow_mut();
        let sens = 0.005;

        let up_vector = rendering_struct.camera.stats.rotation.rotate_vector(Vector3::unit_y());

        let yaw = Rad(-self.mouse_movement_delta.0 * sens);
        let pitch = Rad(-self.mouse_movement_delta.1 * sens);

        let yaw_quaternion = Quaternion::from_axis_angle(up_vector, yaw);
        let pitch_quaternion = Quaternion::from_axis_angle(rendering_struct.camera.stats.rotation*Vector3::unit_x(), pitch);
        let roll_quaternion = Quaternion::from_axis_angle(rendering_struct.camera.stats.rotation*Vector3::unit_z(), Rad(0.0));


        rendering_struct.camera.stats.rotation = roll_quaternion * yaw_quaternion * pitch_quaternion * rendering_struct.camera.stats.rotation;
        self.mouse_movement_delta = (0.0, 0.0);
    }

    pub fn mouse_clicked(&mut self, event: web_sys::PointerEvent) {
        self.mouse_locked = !self.mouse_locked;
        if event.button() != 0 { //mouseEvent.button reference: https://developer.mozilla.org/en-US/docs/Web/API/MouseEvent/button
            return;
        }
        if self.mouse_locked {
            web_sys::window().unwrap()
                .document().unwrap()
                .get_element_by_id(consts::CANVAS_ID).unwrap()
                .request_pointer_lock();
        } else {
            web_sys::window().unwrap()
                .document().unwrap().exit_pointer_lock()
        }
    }
}
