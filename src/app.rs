use cgmath::{Deg, Vector2};
use log::{error, info};
use palette::Srgba;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

use crate::renderer::Renderer;

static FONT_SIZE: f32 = 32.;
static LINE_HEIGHT: f32 = 32.;
static PADDLE_SPEED: f32 = 1000.0;
static BALL_SPEED: f32 = 400.0;
static BALL_RADIUS: f32 = 20.0;

struct State {
    left: Paddle,
    right: Paddle,
    ball: Ball,
    keys_pressed: HashSet<KeyCode>,
    last_update: Instant,
}

struct Paddle {
    score: u8,
    pos: Vector2<f32>,
    width: f32,
    height: f32,
}

struct Ball {
    pos: Vector2<f32>,
    velocity: Vector2<f32>,
    radius: f32,
}

impl Ball {
    fn reset(&mut self, screen_width: f32, screen_height: f32) {
        self.pos = Vector2::new(screen_width / 2.0, screen_height / 2.0);

        // Random direction between -45 and 45 degrees from horizontal
        let angle = rand::random::<f32>() * std::f32::consts::PI / 2.0 - std::f32::consts::PI / 4.0;

        // Randomly choose left or right direction
        let direction = if rand::random::<bool>() { 1.0 } else { -1.0 };

        self.velocity = Vector2::new(
            direction * BALL_SPEED * angle.cos(),
            BALL_SPEED * angle.sin(),
        );
    }
}

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    renderer: Option<Renderer>,
    state: Option<State>,
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
                // Initial ball velocity (moving right and slightly down)
                let initial_velocity = Vector2::new(BALL_SPEED, BALL_SPEED / 3.0);

                self.state = Some(State {
                    left: Paddle {
                        pos: Vector2 {
                            x: 0.0,
                            y: (renderer.size.height / 2) as f32,
                        },
                        score: 0,
                        width: 20.,
                        height: 100.,
                    },
                    right: Paddle {
                        pos: Vector2 {
                            x: renderer.size.width as f32,
                            y: (renderer.size.height / 2) as f32,
                        },
                        score: 0,
                        width: 20.,
                        height: 100.,
                    },
                    ball: Ball {
                        pos: Vector2 {
                            x: (renderer.size.width / 2) as f32,
                            y: (renderer.size.height / 2) as f32,
                        },
                        velocity: initial_velocity,
                        radius: BALL_RADIUS,
                    },
                    keys_pressed: HashSet::new(),
                    last_update: Instant::now(),
                });

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

        let Some(state) = self.state.as_mut() else {
            return info!("Skip window_event handling. We have no state");
        };

        match event {
            WindowEvent::CloseRequested => {
                info!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if let PhysicalKey::Code(key_code) = event.physical_key {
                    match event.state {
                        ElementState::Pressed => {
                            state.keys_pressed.insert(key_code);

                            // Reset the ball if Space is pressed
                            if key_code == KeyCode::Space {
                                state
                                    .ball
                                    .reset(renderer.size.width as f32, renderer.size.height as f32);
                            }
                        }
                        ElementState::Released => {
                            state.keys_pressed.remove(&key_code);
                        }
                    }
                }
            }
            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let delta = now.duration_since(state.last_update).as_secs_f32();
                state.last_update = now;

                // Input Handling:
                {
                    // Left paddle
                    if state.keys_pressed.contains(&KeyCode::KeyW) {
                        state.left.pos.y -= PADDLE_SPEED * delta;
                        if state.left.pos.y < state.left.height / 2. {
                            state.left.pos.y = state.left.height / 2.;
                        }
                    }
                    if state.keys_pressed.contains(&KeyCode::KeyS) {
                        state.left.pos.y += PADDLE_SPEED * delta;
                        if state.left.pos.y > renderer.size.height as f32 - (state.left.height / 2.)
                        {
                            state.left.pos.y =
                                renderer.size.height as f32 - (state.left.height / 2.);
                        }
                    }

                    // Right paddle
                    if state.keys_pressed.contains(&KeyCode::ArrowUp) {
                        state.right.pos.y -= PADDLE_SPEED * delta;
                        if state.right.pos.y < state.right.height / 2. {
                            state.right.pos.y = state.right.height / 2.;
                        }
                    }
                    if state.keys_pressed.contains(&KeyCode::ArrowDown) {
                        state.right.pos.y += PADDLE_SPEED * delta;
                        if state.right.pos.y
                            > renderer.size.height as f32 - (state.right.height / 2.)
                        {
                            state.right.pos.y =
                                renderer.size.height as f32 - (state.right.height / 2.);
                        }
                    }
                }

                // Ball movement
                {
                    state.ball.pos.x += state.ball.velocity.x * delta;
                    state.ball.pos.y += state.ball.velocity.y * delta;

                    // Ball collision with top and bottom walls
                    if state.ball.pos.y - state.ball.radius < 0.0 {
                        state.ball.pos.y = state.ball.radius;
                        state.ball.velocity.y = state.ball.velocity.y.abs(); // Bounce down
                    }
                    if state.ball.pos.y + state.ball.radius > renderer.size.height as f32 {
                        state.ball.pos.y = renderer.size.height as f32 - state.ball.radius;
                        state.ball.velocity.y = -state.ball.velocity.y.abs(); // Bounce up
                    }

                    // Ball collision with left paddle
                    if state.ball.pos.x - state.ball.radius < state.left.pos.x + state.left.width
                        && state.ball.pos.y > state.left.pos.y - state.left.height / 2.0
                        && state.ball.pos.y < state.left.pos.y + state.left.height / 2.0
                    {
                        state.ball.pos.x = state.left.pos.x + state.left.width + state.ball.radius;

                        // Bounce right with angle based on where the ball hit the paddle
                        let relative_intersect_y = state.left.pos.y - state.ball.pos.y;
                        let normalized_relative_intersection_y =
                            relative_intersect_y / (state.left.height / 2.0);
                        let bounce_angle =
                            normalized_relative_intersection_y * std::f32::consts::PI / 4.0;

                        state.ball.velocity.x = BALL_SPEED * bounce_angle.cos();
                        state.ball.velocity.y = -BALL_SPEED * bounce_angle.sin();
                    }

                    // Ball collision with right paddle
                    if state.ball.pos.x + state.ball.radius > state.right.pos.x - state.right.width
                        && state.ball.pos.y > state.right.pos.y - state.right.height / 2.0
                        && state.ball.pos.y < state.right.pos.y + state.right.height / 2.0
                    {
                        state.ball.pos.x =
                            state.right.pos.x - state.right.width - state.ball.radius;

                        // Bounce left with angle based on where the ball hit the paddle
                        let relative_intersect_y = state.right.pos.y - state.ball.pos.y;
                        let normalized_relative_intersection_y =
                            relative_intersect_y / (state.right.height / 2.0);
                        let bounce_angle =
                            normalized_relative_intersection_y * std::f32::consts::PI / 4.0;

                        state.ball.velocity.x = -BALL_SPEED * bounce_angle.cos();
                        state.ball.velocity.y = -BALL_SPEED * bounce_angle.sin();
                    }

                    // Scoring: ball out of bounds
                    if state.ball.pos.x < 0.0 {
                        state.right.score += 1;
                        state
                            .ball
                            .reset(renderer.size.width as f32, renderer.size.height as f32);
                    }
                    if state.ball.pos.x > renderer.size.width as f32 {
                        state.left.score += 1;
                        state
                            .ball
                            .reset(renderer.size.width as f32, renderer.size.height as f32);
                    }
                }

                // Render:
                {
                    renderer.begin_drawing();
                    renderer.clear_color(Srgba::new(0.1, 0.1, 0.1, 1.));

                    // Draw Left
                    renderer.draw_rectangle(
                        Vector2::new(
                            state.left.pos.x,
                            state.left.pos.y - (state.left.height / 2.),
                        ),
                        state.left.width,
                        state.left.height,
                        Srgba::new(1., 0., 0., 1.),
                        Deg(0.),
                    );

                    // Draw Right
                    renderer.draw_rectangle(
                        Vector2::new(
                            state.right.pos.x - (state.right.width),
                            state.right.pos.y - (state.right.height / 2.),
                        ),
                        state.right.width,
                        state.right.height,
                        Srgba::new(0., 0., 1., 1.),
                        Deg(0.),
                    );

                    // Draw Ball
                    renderer.draw_circle(
                        state.ball.pos,
                        state.ball.radius,
                        Srgba::new(1.0, 1.0, 1.0, 1.0),
                    );

                    // Draw center line
                    renderer.draw_rectangle(
                        Vector2::new(renderer.size.width as f32 / 2.0 - 2.0, 0.0),
                        4.0,
                        renderer.size.height as f32,
                        Srgba::new(0.5, 0.5, 0.5, 0.5),
                        Deg(0.),
                    );

                    renderer.draw_text(
                        &format!("P1: {}", state.left.score),
                        Vector2::new(0., 0.),
                        FONT_SIZE,
                        LINE_HEIGHT,
                        None,
                    );

                    let text = "Pong\nGame";
                    let text_width = renderer.measure_text(text, FONT_SIZE, LINE_HEIGHT);
                    renderer.draw_text(
                        text,
                        Vector2::new(renderer.size.width as f32 / 2. - text_width / 2., 0.),
                        FONT_SIZE,
                        LINE_HEIGHT,
                        None,
                    );

                    let text = &format!("P2: {}", state.right.score);
                    let text_width = renderer.measure_text(text, FONT_SIZE, LINE_HEIGHT);
                    renderer.draw_text(
                        text,
                        Vector2::new(renderer.size.width as f32 - text_width, 0.),
                        FONT_SIZE,
                        LINE_HEIGHT,
                        None,
                    );

                    if let Err(err) = renderer.end_drawing() {
                        error!("Error: renderer.render(): {}", err);
                    }
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
