use cgmath::{Vector2, Vector3, Vector4};
use std::ops::Deref;
use std::ops::DerefMut;
macro_rules! vec_impl {
    ($($num:expr,)+) => {
      $(
        paste::paste! {
          #[repr(C)]
          #[derive(Debug, Copy, Clone)]
          pub struct [<Vec $num>](pub [<Vector $num>]<f32>);
          unsafe impl bytemuck::Pod for [<Vec $num>] {}
          unsafe impl bytemuck::Zeroable for [<Vec $num>] {}
          impl From<[<Vector $num>]<f32>> for [<Vec $num>] {
              fn from(v: [<Vector $num>]<f32>) -> Self {
                  Self(v)
              }
          }
          impl Deref for [<Vec $num>] {
              type Target = [<Vector $num>]<f32>;
              fn deref(&self) -> &Self::Target {
                  &self.0
              }
          }
          impl DerefMut for [<Vec $num>] {
              fn deref_mut(&mut self) -> &mut Self::Target {
                  &mut self.0
              }
          }
        }
      )+
    };
}

vec_impl!(2, 3, 4,);

impl Vec2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self(Vector2::new(x, y))
    }
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Self(Vector3::new(x, y, z))
    }
}

impl Vec4 {
    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self(Vector4::new(x, y, z, w))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub pos: Vec2,
    pub color: Vec4,
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x2, 1 => Float32x4];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;

        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<Self>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}
