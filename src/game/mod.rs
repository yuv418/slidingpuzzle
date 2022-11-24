use ggez::event;
use ggez::graphics::{self, Image};
use ggez::{Context, GameResult};
use glam::*;
use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;
use rand::Rng;
use std::path::PathBuf;

pub mod tile;

#[derive(Debug)]
pub struct GameState<'a> {
    pub tile_state: tile::TileState<'a>,
    pub set_winsize: bool,
}

impl GameState<'_> {
    pub fn new(img_path: PathBuf, tile_size: u32, context: &mut Context) -> GameResult<Self> {
        let img = ImageReader::open(img_path)?
            .decode()
            .expect("failed to open image");

        // How many tiles in a row? In a column?
        let col_cnt_tiles = img.width() / tile_size;
        let row_cnt_tiles = img.height() / tile_size;

        // Loop through and make the tiles
        let mut tile_state = tile::TileState {
            tiles: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
        };

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
                let tile_to_insert = tile::Tile {
                    side_len: tile_size,
                    image_buf: Image::from_rgba8(
                        context,
                        tile_size as u16,
                        tile_size as u16,
                        &row_buf_pix,
                    )?,
                };

                let i = i as usize;
                let j = j as usize;

                tile_state.tiles[i][j] = tile_to_insert;
                tile_state.ref_board[i][j] = Some(&tile_state.tiles[i][j]);
                tile_row.push(tile_to_insert);
            }
        }

        // Remove one random tile from ref board.
        let mut rng = rand::thread_rng();

        // scramble the tiles in tile_state.tile_order
        let i = rng.gen_range(0..row_cnt_tiles) as usize;
        let j = rng.gen_range(0..col_cnt_tiles) as usize;
        tile_state.ref_board[i][j] = None;

        Ok(Self {
            tile_state,
            set_winsize: false,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState<'_> {
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        keycode: event::KeyCode,
        _keymods: event::KeyMods,
        repeat: bool,
    ) {
        if !repeat {
            println!("key down event {:?}", keycode);
        }
    }
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        // self.pos_x = self.pos_x % 800.0 + 20.0;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        // draw like one tile

        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        // draw all tiles with a 10px gap between each title
        for i in 0..self.tile_state.ref_board.len() {
            // each tile in the row, so x
            for j in 0..self.tile_state.tiles[i].len() {
                let tile = &self.tile_state.tiles[i][j];
                if let Some(&tile) = tile {
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
        }
        graphics::present(ctx)?;

        Ok(())
    }
}
