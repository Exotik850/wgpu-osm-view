use glam::{Mat4, Vec2, Vec3, Vec3Swizzles};

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

    // (width, height) -> (1, 1)
    pub fn screen_to_world(&self, screen_pos: Vec2, screen_size: Vec2) -> Vec2 {
        let screen_pos = Vec2::new(screen_pos.x as f32, screen_size.y - screen_pos.y as f32); // Flip Y-axis
        let normalized_pos = (screen_pos / screen_size) * 2.0 - Vec2::ONE; // Convert to [-1, 1] range
        (normalized_pos - self.offset) / self.zoom
    }

    pub fn matrix(&self) -> Mat4 {
        let translation = Mat4::from_translation(Vec3::new(self.offset.x, self.offset.y, 0.0));
        let scaling = Mat4::from_scale(Vec3::new(self.zoom, self.zoom, 1.0));
        translation * scaling
    }
}

pub struct CameraController {
    pub camera: Camera,
    screen_size: Vec2,
    mouse_pos: Vec2,
    mouse_down: bool,
    scroll_velocity: f32,
}

impl CameraController {
    pub fn new(screen_size: Vec2) -> Self {
        Self {
            camera: Camera::new(),
            screen_size,
            mouse_pos: Vec2::new(0.0, 0.0),
            mouse_down: false,
            scroll_velocity: 0.0,
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
        // self.camera.zoom += self.scroll_velocity * 0.1;
    }
    pub fn mouse_down(&mut self, down: bool) {
        self.mouse_down = down;
    }

    // pub fn scroll(&mut self, amount: f32) {
    //     self.scroll_velocity = amount;
    //     let mouse_pos = self
    //         .matrix()
    //         .inverse()
    //         .transform_vector3(self.mouse_pos.extend(0.0))
    //         .xy();
    //     self.camera.zoom *= 1.0 + amount * 0.1;
    //     let new_mouse_pos = self
    //         .matrix()
    //         // .inverse()
    //         .transform_point3(Vec3::new(mouse_pos.x, mouse_pos.y, 0.0))
    //         .xy();
    //     self.camera.offset += self.mouse_pos - new_mouse_pos;
    // }

    pub fn scroll(&mut self, amount: f32) {
      // Pre-calculate zoom factor
      let zoom_factor = 1.0 + amount * 0.1;
      let new_zoom = self.camera.zoom * zoom_factor;
  
      // Calculate the mouse position in world space before zooming
      let inv_zoom = 1.0 / self.camera.zoom;
      let mouse_world_x = (self.mouse_pos.x - self.camera.offset.x) * inv_zoom;
      let mouse_world_y = (self.mouse_pos.y - self.camera.offset.y) * inv_zoom;
  
      // Update zoom
      self.camera.zoom = new_zoom;
  
      // Calculate new offset
      let new_mouse_screen_x = mouse_world_x * new_zoom + self.camera.offset.x;
      let new_mouse_screen_y = mouse_world_y * new_zoom + self.camera.offset.y;
  
      // Update offset
      self.camera.offset.x += self.mouse_pos.x - new_mouse_screen_x;
      self.camera.offset.y += self.mouse_pos.y - new_mouse_screen_y;
  }

  //   pub fn scroll(&mut self, amount: f32) {
  //     let mat = self.matrix();
  //     let inv_mat = mat.inverse();
  //     let mouse_pos =
  //         inv_mat.transform_point3(Vec3::new(self.mouse_pos.x, self.mouse_pos.y, 0.0));
  //     let zoom = self.camera.zoom * (1.0 + amount * 0.1);
  //     self.camera.zoom = zoom;
  //     let new_mat = self.matrix();
  //     let new_mouse_pos = new_mat.transform_point3(Vec3::new(mouse_pos.x, mouse_pos.y, 0.0));
  //     self.camera.offset.x += self.mouse_pos.x - new_mouse_pos.x;
  //     self.camera.offset.y += self.mouse_pos.y - new_mouse_pos.y;
  // }

    pub fn apply_velocity(&mut self) {
        if self.scroll_velocity.abs() > 0.01 {
            self.scroll(self.scroll_velocity);
            self.scroll_velocity *= 0.7;
        }
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        self.camera.screen_to_world(screen_pos, self.screen_size)
    }

    pub fn resize(&mut self, size: Vec2) {
        self.screen_size = size;
    }

    pub fn matrix(&self) -> Mat4 {
        self.camera.matrix()
    }
}
