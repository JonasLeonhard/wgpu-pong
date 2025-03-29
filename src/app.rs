use cgmath::{Deg, Vector2};
use log::{error, info};
use palette::Srgba;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

use crate::renderer::Renderer;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = match event_loop.create_window(Window::default_attributes()) {
            Ok(window) => Arc::new(window),
            Err(err) => {
                return error!("Failed to create window: {}", err);
            }
        };

        match pollster::block_on(Renderer::new(window.clone())) {
            Ok(renderer) => {
                self.renderer = Some(renderer);
            }
            Err(err) => {
                error!("Failed to create renderer: {}", err);
            }
        }

        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let Some(window) = self.window.as_ref() else {
            return info!("Skip window_event handling. We have no window");
        };

        let Some(renderer) = self.renderer.as_mut() else {
            return info!("Skip window_event handling. We have no renderer");
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                renderer.begin_drawing();

                // Draw
                renderer.draw_rectangle(
                    Vector2::new(
                        renderer.size.width as f32 / 3.,
                        renderer.size.height as f32 / 3.,
                    ),
                    renderer.size.width as f32 / 3.,
                    renderer.size.height as f32 / 3.,
                    Srgba::new(1., 0., 0., 1.),
                    Deg(45.),
                );

                renderer.draw_triangle(
                    Vector2::new(
                        renderer.size.width as f32 / 2. - 200.,
                        renderer.size.height as f32 / 2. - 200.,
                    ),
                    Vector2::new(
                        renderer.size.width as f32 / 2.,
                        renderer.size.height as f32 / 2. + 200.,
                    ),
                    Vector2::new(
                        renderer.size.width as f32 / 2. + 200.,
                        renderer.size.height as f32 / 2. - 200.,
                    ),
                    Srgba::new(0., 1., 0., 1.),
                    Deg(90.),
                );

                if let Err(err) = renderer.end_drawing() {
                    error!("Error: renderer.render(): {}", err);
                }

                window.request_redraw();
            }
            WindowEvent::Resized(size) => {
                // Reconfigures the size of the surface. We do not re-render
                // here as this event is always followed up by redraw request.
                renderer.resize(size);
            }
            _ => (),
        }
    }
}
