use std::{
    collections::HashMap,
    os::raw,
    path::{Path, PathBuf},
};

use anyhow::Result;
use compression::prelude::*;
use glam::{Vec2, Vec3, Vec4};
use graphics::Graphics;
use pollster::FutureExt;
use radix_trie::Trie;
use vertex::Vertex;
use winit::{
    event::{Event, WindowEvent},
    event_loop::EventLoop,
    window::WindowBuilder,
};
mod camera;
mod graphics;
mod osm;
mod shaders;
mod vertex;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct RawRenderData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl RawRenderData {
    pub fn new(vertices: Vec<Vertex>, indices: Vec<u32>) -> Self {
        Self { vertices, indices }
    }

    pub fn from_osm(osm: &osm::OSM) -> Self {
        let vertices = osm.vertices();
        let indices = osm.indices();
        Self::new(vertices, indices)
    }

    pub fn cache_to(&self, cache_path: impl AsRef<Path>) -> Result<()> {
        if !cache_path.as_ref().exists() {
            let bytes = bincode::serialize(&self)?;
            let bytes: Vec<_> = bytes
                .into_iter()
                .encode(&mut ZlibEncoder::new(), Action::Finish)
                .collect::<Result<_, _>>()?;
            std::fs::write(cache_path, bytes)?;
        }
        Ok(())
    }

    pub fn sorted(&self, cell_size: f32) -> SortedRenderData {
        let mut map = HashMap::new();
        for (i, vertex) in self.vertices.iter().enumerate() {
            let x = (vertex.pos.x / cell_size).floor() as i32;
            let y = (vertex.pos.y / cell_size).floor() as i32;
            map.entry((x, y)).or_insert_with(Vec::new).push(i);
        }

        SortedRenderData {
            raw_ref: self,
            cell_size,
            map,
        }
    }
}

pub struct SortedRenderData<'a> {
    raw_ref: &'a RawRenderData,
    cell_size: f32,
    map: HashMap<(i32, i32), Vec<usize>>,
}

impl<'a> SortedRenderData<'a> {
    pub fn get(&self, x: f32, y: f32) -> Option<&[usize]> {
        self.map.get(&self.convert(x, y)).map(|v| v.as_slice())
    }

    pub fn get_vec(&self, pos: Vec2) -> Option<&[usize]> {
        self.map
            .get(&self.convert(pos.x, pos.y))
            .map(|v| v.as_slice())
    }

    pub fn convert(&self, x: f32, y: f32) -> (i32, i32) {
        (
            (x / self.cell_size).floor() as i32,
            (y / self.cell_size).floor() as i32,
        )
    }
}

fn main() -> Result<()> {
    let mut args = std::env::args();
    args.next();
    let osm_path = args.next().expect("No OSM file provided");
    // let cache_path = Path::new("./cache.bin");
    let cache_path = PathBuf::from(format!("{}.cache", osm_path));
    let raw_render_data = if !cache_path.exists() {
        let osm = osm::OSM::load(osm_path)?;
        let raw_render_data = RawRenderData::from_osm(&osm);
        // std::fs::write(cache_path, bincode::serialize(&raw_render_data)?)?;
        raw_render_data
    } else {
        let bytes = std::fs::read(&cache_path)?;
        let bytes: Vec<_> = bytes
            .into_iter()
            .decode(&mut ZlibDecoder::new())
            .collect::<Result<_, _>>()?;
        bincode::deserialize(&bytes)?
    };

    // let vertices = raw_render_data.vertices.clone();
    // let sorted = raw_render_data.sorted(0.1);

    // let vertices = vec![
    //     Vertex::new(Vec2::new(0.0, 0.0)),
    //     Vertex::new(Vec2::new(1.0, 0.0)),
    //     Vertex::new(Vec2::new(1.0, 1.0)),
    // ];
    // let indices = vec![0, 1, u32::MAX, 2, 0];

    // println!("Loaded {} points", vertices.len());
    // let stops = indices.iter().filter(|i| **i == std::u32::MAX).count();
    // println!("Loaded {} lines", stops);
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new()
        .with_title("WGPU OSM View")
        .build(&event_loop)?;
    let mut graphics =
        Graphics::new(window, &raw_render_data.vertices, &raw_render_data.indices).block_on()?;
    // let mut current_points = Vec::new();
    // let mut world_pos = Vec2::new(0.0, 0.0);
    let mut c_controller = camera::CameraController::new(graphics.size_vec());
    event_loop.run(move |event, control_flow| {
        if graphics.input(&event) {
            return;
        }

        match event {
            Event::WindowEvent {
                event, window_id, ..
            } if window_id == graphics.window().id() => match event {
                WindowEvent::CloseRequested => control_flow.exit(),
                WindowEvent::RedrawRequested => {
                    c_controller.apply_velocity();
                    // graphics.update(&c_controller, &current_points);
                    graphics.update(&c_controller);
                    // if let Some(points) = sorted.get_vec(world_pos) {
                    //     // println!("Found {} points", points.len());
                    //     current_points = points.iter().map(|i| vertices[*i]).collect();
                    // } else {
                    //     // println!("No points found in this cell");
                    // }
                    graphics.render();
                }
                WindowEvent::Resized(new_size) => {
                    graphics.resize(new_size);
                    c_controller.resize(graphics.size_vec());
                }
                WindowEvent::CursorMoved { position, .. } => {
                    let size = graphics.size_vec();
                    let screen_pos = Vec2::new(position.x as f32, position.y as f32);
                    c_controller.update(screen_pos, size);
                    // world_pos = screen_pos / size;
                    // println!("World pos: {:?}", screen_pos / size);
                }
                WindowEvent::MouseInput { state, button, .. } => {
                    c_controller.mouse_down(
                        state == winit::event::ElementState::Pressed
                            && button == winit::event::MouseButton::Left,
                    );

                    // Reset camera if right mouse button is pressed
                    if state == winit::event::ElementState::Pressed
                        && button == winit::event::MouseButton::Right
                    {
                        c_controller.camera = camera::Camera::new();
                    }
                }
                WindowEvent::MouseWheel { delta, .. } => {
                    let amt = match delta {
                        winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                        winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32,
                    };
                    c_controller.scroll(amt);
                }
                _ => (),
            },
            Event::AboutToWait => {
                graphics.window().request_redraw();
            }
            _ => (),
        }
    })?;

    raw_render_data.cache_to(cache_path)?;

    Ok(())
}
