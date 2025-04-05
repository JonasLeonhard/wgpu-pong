## WGPU Pong

[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://choosealicense.com/licenses/mit/)

WGPU renderer pipeline for pong. The renderer implements a raylib like api for rendering.

```rust
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

```

## Run Locally

```bash
cargo run
```
