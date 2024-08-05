use crate::vertex::Vec2;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Camera {
    pub offset: Vec2,
    pub zoom: f32,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            offset: Vec2::new(0.0, 0.0),
            zoom: 1.0,
        }
    }
}

pub struct CameraController {
    pub camera: Camera,
    screen_size: Vec2,
    mouse_pos: Vec2,
    mouse_down: bool,
}

impl CameraController {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            camera: Camera::new(),
            screen_size,
            mouse_pos: Vec2::new(0.0, 0.0),
            mouse_down: false,
        }
    }

    pub fn update(&mut self, position: Vec2, size: Vec2) {
        let new_pos = Vec2::new(
            (position.x / size.x) * 2.0 - 1.0,
            1.0 - (position.y / size.y) * 2.0,
        );
        if self.mouse_down {
            // Calculate the difference in normalized space
            let delta = Vec2::new(new_pos.x - self.mouse_pos.x, new_pos.y - self.mouse_pos.y);

            // Adjust the camera offset
            self.camera.offset.x += delta.x;
            self.camera.offset.y += delta.y;
        }
        self.mouse_pos = new_pos;
    }
    pub fn mouse_down(&mut self, down: bool) {
        self.mouse_down = down;
    }

    pub fn scroll(&mut self, amount: f32) {
        let zoom_factor = 1.0 + amount * 0.1; // Adjust 0.1 to control zoom speed
        let old_zoom = self.camera.zoom;
        let new_zoom = self.camera.zoom * zoom_factor;

        // Calculate the scale change
        let scale_change = new_zoom - old_zoom;

        // Adjust the offset based on the zoom point (mouse position)
        self.camera.offset.x -= self.mouse_pos.x * scale_change;
        self.camera.offset.y -= self.mouse_pos.y * scale_change;

        // Update the zoom
        self.camera.zoom = new_zoom;
    }

    pub fn resize(&mut self, size: Vec2) {
        self.screen_size = size;
    }
}
