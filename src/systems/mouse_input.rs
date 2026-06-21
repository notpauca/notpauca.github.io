use cgmath::Deg;
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
        rendering_struct.camera.stats.pitch += Deg(-self.mouse_movement_delta.1).into();
        rendering_struct.camera.stats.yaw += Deg(self.mouse_movement_delta.0).into();
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
