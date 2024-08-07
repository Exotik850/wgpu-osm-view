use glam::{Mat4, Vec2, Vec3};

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

    pub fn matrix(&self) -> Mat4 {
        // Translation to move the camera offset to the center
        let translation = Mat4::from_translation(Vec3::new(self.offset.x, self.offset.y, 0.0));

        // Scaling for zoom
        let scaling = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));

        // Combine scaling and translation to create the final transformation matrix
        translation * scaling
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
        let mat = self.matrix();
        let inv_mat = mat.inverse();
        let mouse_pos =
            inv_mat.transform_point3(Vec3::new(self.mouse_pos.x, self.mouse_pos.y, 0.0));
        let zoom = self.camera.zoom * (1.0 + amount * 0.1);
        self.camera.zoom = zoom;
        let new_mat = self.matrix();
        let new_mouse_pos = new_mat.transform_point3(Vec3::new(mouse_pos.x, mouse_pos.y, 0.0));
        self.camera.offset.x += self.mouse_pos.x - new_mouse_pos.x;
        self.camera.offset.y += self.mouse_pos.y - new_mouse_pos.y;
    }

    pub fn resize(&mut self, size: Vec2) {
        self.screen_size = size;
    }

    pub fn matrix(&self) -> Mat4 {
        self.camera.matrix()
    }
}
