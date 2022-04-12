use ggez::event;
use ggez::graphics::Image;
use ggez::graphics::Rect;
use ggez::graphics::{self, Color};
use ggez::{Context, GameResult};
use glam::*;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;
use image::Rgba;
use std::io::Cursor;
use std::path::PathBuf;

// Turn the input image into a 2d array,
// expect it's a square, and then divide the image
// into X by X tiles

#[derive(Debug)]
struct Tile {
    // The size of a square tile (one side) in px
    side_len: u32,
    image_buf: Image,
}

#[derive(Debug)]
struct TileState {
    tiles: Vec<Vec<Tile>>,
    tile_order: Vec<Vec<(u32, u32)>>, // 2d array of tile positions for rendering when a user plays
}

#[derive(Debug)]
struct MainState {
    tile_state: TileState,
    set_winsize: bool,
}

impl MainState {
    fn new(img_path: PathBuf, tile_size: u32, context: &mut Context) -> GameResult<MainState> {
        let img = ImageReader::open(img_path)?
            .decode()
            .expect("failed to open image");

        // Loop through and make the tiles
        let mut tile_state = TileState {
            tiles: vec![],
            tile_order: vec![],
        };

        // How many tiles in a row? In a column?
        let col_cnt_tiles = img.width() / tile_size;
        let row_cnt_tiles = img.height() / tile_size;

        // Go through each row of tiles, looping through each tile in the row
        for i in 1..(row_cnt_tiles + 1) {
            let mut tile_row = vec![];
            for j in 1..(col_cnt_tiles + 1) {
                // Go through each square block, adding a tile to our TileState
                let mut row_buf_pix = vec![];
                for y in (tile_size * (i - 1))..(tile_size * i) {
                    // add each row individiaully to the tile_to_insert
                    // row buffer of pixels
                    for x in (tile_size * (j - 1))..(tile_size * j) {
                        let pix_rgba = img.get_pixel(x, y).to_rgba().0;
                        for rgba_val in pix_rgba {
                            row_buf_pix.push(rgba_val);
                        }
                    }
                    println!("one-row length is {:?}", row_buf_pix.len());
                }
                println!("writing tile to tile_row");
                let tile_to_insert = Tile {
                    side_len: tile_size,
                    image_buf: Image::from_rgba8(
                        context,
                        tile_size as u16,
                        tile_size as u16,
                        &row_buf_pix,
                    )?,
                };
                tile_row.push(tile_to_insert);
            }
            tile_state.tiles.push(tile_row);
        }

        // scramble the tiles in tile_state.tile_order

        Ok(MainState {
            tile_state,
            set_winsize: false,
        })
    }
}

impl event::EventHandler<ggez::GameError> for MainState {
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.pos_x = self.pos_x % 800.0 + 20.0;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        // draw like one tile

        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        // draw all tiles with a 10px gap between each title
        for i in 0..self.tile_state.tiles.len() {
            // each tile in the row, so x
            for j in 0..self.tile_state.tiles[i].len() {
                let tile = &self.tile_state.tiles[i][j];
                let tile_gap = tile.side_len + 10; // determine the gap here

                graphics::draw(
                    ctx,
                    &tile.image_buf,
                    (Vec2::new(
                        (j as u32 * tile_gap) as f32 + 90.0,
                        (i as u32 * tile_gap) as f32 + 150.0,
                    ),),
                )?;
            }
        }
        graphics::present(ctx)?;

        Ok(())
    }
}

pub fn main() -> GameResult {
    let (mut ctx, event_loop) = ggez::ContextBuilder::new("SlidingPuzzle", "cdknight")
        .window_setup(ggez::conf::WindowSetup::default().title("Sliding Puzzle"))
        .window_mode(
            ggez::conf::WindowMode::default()
                .min_dimensions(100.0, 100.0)
                .resizable(true),
        )
        .build()?;

    let state = MainState::new("test.jpg".into(), 120, &mut ctx)?;

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
    let tile_gap = state.tile_state.tiles[0][0].side_len + 10; // determine the gap here
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
