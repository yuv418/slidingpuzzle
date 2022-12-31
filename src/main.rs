use std::path::PathBuf;

use ggez::{
    conf::{FullscreenType, WindowMode},
    event,
    winit::dpi::LogicalSize,
    GameResult,
};

mod game;

// Turn the input image into a 2d array,
// expect it's a square, and then divide the image
// into X by X tiles

pub fn main() -> GameResult {
    env_logger::init();

    let mut winmode = ggez::conf::WindowMode::default();

    if let Some("fullscreen") = std::env::args().nth(1).as_deref() {
        winmode = winmode.fullscreen_type(FullscreenType::Desktop);
    } else {
        winmode = winmode.dimensions(1820.0, 1030.0);
    }

    let mut cb = ggez::ContextBuilder::new("SlidingPuzzle", "cdknight")
        .window_setup(ggez::conf::WindowSetup::default().title("Sliding Puzzle"))
        .window_mode(winmode);
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }
    let (mut ctx, event_loop) = cb.build()?;

    if let Some("fullscreen") = std::env::args().nth(1).as_deref() {
        ctx.gfx.set_mode(WindowMode {
            fullscreen_type: FullscreenType::Desktop,
            logical_size: Some(LogicalSize::from([ctx.gfx.drawable_size().0, 1030.0])),
            ..Default::default()
        })?;
    }

    /*
             180 px top padding
      60 + [        width        ] + 60
      60 + [        width        ] + 60
      60 + [        width        ] + 60
      60 + [        width        ] + 60
      60 + [        width        ] + 60
      60 + [        width        ] + 60
           180 px bottom padding
    */
    // you have to set the window to resizable first before you can resize it.
    //
    //
    // copied straight from https://github.com/ggez/ggez/blob/master/examples/files.rs

    // Drop the mutex by putting it in its own scope
    let state = game::GameState::new(&mut ctx)?;

    // ctx.gfx.set_screen_coordinates(&mut ctx, Rect::new(0.0, 0.0, win_width, win_height))?

    event::run(ctx, event_loop, state)
}
