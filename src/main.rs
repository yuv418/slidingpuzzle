use std::path::PathBuf;

use ggez::event;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

mod game;

// Turn the input image into a 2d array,
// expect it's a square, and then divide the image
// into X by X tiles

pub fn main() -> GameResult {
    let mut cb = ggez::ContextBuilder::new("SlidingPuzzle", "cdknight")
        .window_setup(ggez::conf::WindowSetup::default().title("Sliding Puzzle"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .min_dimensions(100.0, 100.0)
                .resizable(true),
        );
    if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let mut path = PathBuf::from(manifest_dir);
        path.push("resources");
        println!("Adding path {:?}", path);
        cb = cb.add_resource_path(path);
    }
    let (mut ctx, event_loop) = cb.build()?;

    let state = game::GameState::new("test.jpg".into(), 180, &mut ctx)?;

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
    /*let tile_gap = state.tile_state.tiles[0][0].borrow().side_len + 10; // determine the gap here
    let win_width = (180 + (state.tile_state.tiles[0].len() as u32 * tile_gap)) as f32;
    let win_height = (300 + (tile_gap * state.tile_state.tiles.len() as u32)) as f32;
    println!("the new window dimensions are {}x{}", win_width, win_height);*/
    // you have to set the window to resizable first before you can resize it.
    //
    //
    // copied straight from https://github.com/ggez/ggez/blob/master/examples/files.rs

    ctx.gfx.add_font(
        "SecularOne-Regular",
        graphics::FontData::from_path(&ctx, "/fonts/SecularOne-Regular.ttf")?,
    );

    /*ctx.gfx.set_mode(
        ggez::conf::WindowMode::default()
            .dimensions(win_width, win_height)
            .resizable(true),
    );*/
    // ctx.gfx.set_screen_coordinates(&mut ctx, Rect::new(0.0, 0.0, win_width, win_height))?

    event::run(ctx, event_loop, state)
}
