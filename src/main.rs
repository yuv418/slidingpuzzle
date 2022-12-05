use std::path::PathBuf;

use game::player::{Player, PLAYER};
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
                .min_dimensions(810.0, 830.0)
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
    let intro = {
        let mut opt_player = PLAYER.lock().unwrap();
        let loaded_player = Player::load(&mut ctx);

        match loaded_player {
            Err(_) => true,
            Ok(p) => {
                *opt_player = Some(p);
                false
            }
        }
    };
    let state = game::GameState::new(&mut ctx, intro)?;

    // ctx.gfx.set_screen_coordinates(&mut ctx, Rect::new(0.0, 0.0, win_width, win_height))?

    event::run(ctx, event_loop, state)
}
