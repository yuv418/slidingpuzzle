use ggez::event;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};

mod game;

// Turn the input image into a 2d array,
// expect it's a square, and then divide the image
// into X by X tiles

pub fn main() -> GameResult {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("SlidingPuzzle", "cdknight")
        .window_setup(ggez::conf::WindowSetup::default().title("Sliding Puzzle"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .min_dimensions(100.0, 100.0)
                .resizable(true),
        )
        .build()?;

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
    let tile_gap = state.tile_state.tiles[0][0].borrow().side_len + 10; // determine the gap here
    let win_width = (180 + (state.tile_state.tiles[0].len() as u32 * tile_gap)) as f32;
    let win_height = (300 + (tile_gap * state.tile_state.tiles.len() as u32)) as f32;
    println!("the new window dimensions are {}x{}", win_width, win_height);
    // you have to set the window to resizable first before you can resize it.
    graphics::set_mode(
        &mut ctx,
        ggez::conf::WindowMode::default()
            .dimensions(win_width, win_height)
            .resizable(true),
    )?;
    graphics::set_screen_coordinates(&mut ctx, Rect::new(0.0, 0.0, win_width, win_height))?;
    graphics::set_resizable(&mut ctx, false);

    event::run(ctx, event_loop, state)
}
