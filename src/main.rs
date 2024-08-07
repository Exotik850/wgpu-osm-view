
use anyhow::Result;
use glam::Vec2;
use graphics::Graphics;
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

fn main() -> Result<()> {
    let event_loop = EventLoop::new()?;
    let window = WindowBuilder::new().with_title(" ").build(&event_loop)?;

    let osm = osm::OSM::load("./tennessee-latest.osm.pbf")?;
    let vertices = osm.vertices();
    let indices = osm.indices();

    // let vertices = vec![
    //     Vertex::new(Vec2::new(0.0, 0.0)),
    //     Vertex::new(Vec2::new(1.0, 0.0)),
    //     Vertex::new(Vec2::new(1.0, 1.0)),
    // ];
    // let indices = vec![0, 1, u32::MAX, 2, 0];

    println!("Loaded {} points", vertices.len());
    let stops = indices.iter().filter(|i| **i == std::u32::MAX).count();
    println!("Loaded {} lines", stops);

    let mut graphics = pollster::block_on(Graphics::new(window, vertices, &indices))?;

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
                    graphics.update(&c_controller);
                    graphics.render();
                }
                WindowEvent::Resized(new_size) => {
                    graphics.resize(new_size);
                    c_controller.resize(graphics.size_vec());
                }
                WindowEvent::CursorMoved { position, .. } => {
                    c_controller.update(
                        Vec2::new(position.x as f32, position.y as f32),
                        graphics.size_vec(),
                    );
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

    Ok(())
}
