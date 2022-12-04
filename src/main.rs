use std::path::PathBuf;

use game::player;
use ggez::{event, graphics, GameResult};

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

    ctx.gfx.add_font(
        "SecularOne-Regular",
        graphics::FontData::from_path(&ctx, "/fonts/SecularOne-Regular.ttf")?,
    );

    let state = game::GameState::new(&mut ctx)?;

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

    // Save file
    let player = player::Player::load(&mut ctx);
    if let Err(_) = player {
        // TODO scene for inputting player name
        let p = player::Player::new("test".to_string());
        p.save(&mut ctx)?;
    } else if let Ok(player) = player {
        println!("{:?}", player);
    }

    // ctx.gfx.set_screen_coordinates(&mut ctx, Rect::new(0.0, 0.0, win_width, win_height))?

    event::run(ctx, event_loop, state)
}
