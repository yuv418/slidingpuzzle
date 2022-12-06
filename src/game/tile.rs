use image::{io::Reader as ImageReader, GenericImageView, Pixel};
use rand::Rng;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ggez::{
    graphics::{self, Image},
    Context, GameResult,
};
use glam::Vec2;

use crate::game::drawable::Drawable;

#[derive(Debug, Clone)]
pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
    pub animate_from: Option<(usize, usize, Option<u32>)>,
}

impl Tile {
    pub fn adjacent((i1, j1): (usize, usize), (i2, j2): (usize, usize)) -> bool {
        let (i1, j1) = (i1 as i32, j1 as i32);
        let (i2, j2) = (i2 as i32, j2 as i32);
        if ((i1 - i2).abs() == 1 && (j1 - j2) == 0) || ((i1 - i2) == 0 && (j1 - j2).abs() == 1) {
            true
        } else {
            false
        }
    }
}
impl Drawable for Tile {
    fn draw(&mut self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        graphics::draw(ctx, &self.image_buf, (Vec2::new(x, y),))
    }
}

#[derive(Debug)]
pub struct TileState {
    pub tiles: Vec<Vec<Rc<RefCell<Tile>>>>,
    pub ref_board: Vec<Vec<Option<Rc<RefCell<Tile>>>>>,
    // For efficiency purposes
    pub blank_cell: (usize, usize),
    pub drawing_animation: bool,
    game_completed: bool,
}

impl TileState {
    pub fn new(ctx: &mut Context, img_path: PathBuf, tile_size: u32) -> GameResult<Self> {
        let img = ImageReader::open(img_path)?
            .decode()
            .expect("failed to open image");

        // How many tiles in a row? In a column?
        let col_cnt_tiles = img.width() / tile_size;
        let row_cnt_tiles = img.height() / tile_size;

        // Loop through and make the tiles
        let mut tile_state = Self {
            tiles: vec![],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            blank_cell: (0, 0),
            drawing_animation: false,
            game_completed: false,
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
                let tile_to_insert = Tile {
                    side_len: tile_size,
                    image_buf: Image::from_rgba8(
                        ctx,
                        tile_size as u16,
                        tile_size as u16,
                        &row_buf_pix,
                    )?,
                    animate_from: None,
                };
                tile_row.push(Rc::new(RefCell::new(tile_to_insert)));
            }

            tile_state.tiles.push(tile_row);
        }
        for i in 0..row_cnt_tiles {
            for j in 0..col_cnt_tiles {
                let i = i as usize;
                let j = j as usize;
                tile_state.ref_board[i][j] = Some(tile_state.tiles[i][j].clone());
            }
        }

        let mut rng = rand::thread_rng();

        // Remove one random tile from ref board.
        let i = rng.gen_range(0..row_cnt_tiles) as usize;
        let j = rng.gen_range(0..col_cnt_tiles) as usize;
        tile_state.ref_board[i][j] = None;
        tile_state.blank_cell = (i, j);

        // Scramble the tiles in tile_state.ref_board
        for _ in 1..400 {
            // This is so low effort
            // Better method: start with gap at 0,0 and swap the gap with random adjacents over and over
            let tile1 = (
                rng.gen_range(0..row_cnt_tiles) as usize,
                rng.gen_range(0..col_cnt_tiles) as usize,
            );
            // Choose a random adjacent tile
            let replacetile = if tile_state.blank_cell == (0, 0) {
                rng.gen_range(0..2) as usize
            } else if tile_state.blank_cell == (0, col_cnt_tiles as usize - 1) {
                let r = rng.gen_range(0..2);
                (if r == 1 { 2 } else { r } as usize)
            } else if tile_state.blank_cell
                == (row_cnt_tiles as usize - 1, col_cnt_tiles as usize - 1)
            {
                let r = rng.gen_range(0..2);
                (if r == 0 { 3 } else { 2 } as usize)
            } else if tile_state.blank_cell == (row_cnt_tiles as usize - 1, 0) {
                let r = rng.gen_range(0..2);
                (if r == 0 { 3 } else { r } as usize)
            }
            // Left edge
            else if tile_state.blank_cell.1 == 0 {
                let r = rng.gen_range(0..3);
                (if r == 2 { 3 } else { r } as usize)
            }
            // Top edge
            else if tile_state.blank_cell.0 == 0 {
                rng.gen_range(0..3)
            }
            // Right edge
            else if tile_state.blank_cell.1 == col_cnt_tiles as usize - 1 {
                let r = rng.gen_range(0..3);
                (if r == 1 { 3 } else { r } as usize)
            }
            // Bottom edge
            else if tile_state.blank_cell.0 == row_cnt_tiles as usize - 1 {
                rng.gen_range(0..3) + 1
            } else {
                rng.gen_range(0..4) as usize
            };

            let (c1, c2) = tile_state.blank_cell;

            println!("swap {:?} {:?}", tile_state.blank_cell, replacetile);
            let tile2 = match replacetile {
                0 => {
                    // Down
                    (c1 + 1, c2)
                }
                1 => {
                    // Right
                    (c1, c2 + 1)
                }
                2 => {
                    // Left
                    (c1, c2 - 1)
                }
                3 => {
                    // Up
                    (c1 - 1, c2)
                }
                _ => panic!("Should never happen"),
            };

            tile_state.swap_ref_tiles(tile_state.blank_cell, tile2, false)
        }
        Ok(tile_state)
    }
    pub fn swap_ref_tiles(
        &mut self,
        (i1, j1): (usize, usize),
        (i2, j2): (usize, usize),
        animate: bool,
    ) {
        let old_tile = self.ref_board[i1][j1].clone();
        let swapped_with_blank = if let None = old_tile {
            self.blank_cell = (i2, j2);
            true
        } else {
            false
        };

        self.ref_board[i1][j1] = self.ref_board[i2][j2].clone();
        self.ref_board[i2][j2] = old_tile;

        if animate {
            self.ref_board[i1][j1]
                .as_ref()
                .unwrap()
                .borrow_mut()
                .animate_from = Some((i2, j2, None));
        }
    }

    pub fn check_completed(&mut self) {
        for i in 0..self.ref_board.len() {
            for j in 0..self.ref_board[i].len() {
                if let Some(tile) = &self.ref_board[i][j] {
                    if !Rc::ptr_eq(&self.tiles[i][j], tile) {
                        self.game_completed = false;
                        return;
                    }
                }
            }
        }
        self.game_completed = true
    }

    pub fn game_completed(&self) -> bool {
        self.game_completed
    }
}

impl Drawable for TileState {
    fn draw(&mut self, ctx: &mut Context, x: f32, y: f32) -> GameResult {
        // draw like one tile

        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        // draw all tiles with a 10px gap between each title
        for i in 0..self.ref_board.len() {
            // each tile in the row, so x
            for j in 0..self.ref_board[i].len() {
                let mut tile = if !self.game_completed() {
                    if let Some(tile) = &self.ref_board[i][j] {
                        tile
                    } else {
                        continue;
                    }
                } else {
                    &self.tiles[i][j]
                }
                .borrow_mut();

                let tile_gap = tile.side_len + 10; // determine the gap here
                if let Some((i1, j1, mut pos)) = tile.animate_from {
                    if let None = self.ref_board[i1][j1] {
                        if Tile::adjacent((i, j), (i1, j1)) {
                            if j > j1 {
                                // Move right
                                println!("Animate move right {:?}", tile.animate_from);
                                if let None = tile.animate_from.unwrap().2 {
                                    tile.animate_from = Some((i1, j1, Some(j1 as u32 * tile_gap)));
                                    self.drawing_animation = true;
                                } else if pos < Some(j as u32 * tile_gap) {
                                    tile.animate_from = Some((i1, j1, Some(pos.unwrap() + 20)));
                                } else {
                                    tile.animate_from = None;
                                    self.drawing_animation = false;
                                }

                                if let Some(_) = tile.animate_from {
                                    graphics::draw(
                                        ctx,
                                        &tile.image_buf,
                                        (Vec2::new(
                                            tile.animate_from.unwrap().2.unwrap() as f32 + 90.0,
                                            (i as u32 * tile_gap) as f32 + 150.0,
                                        ),),
                                    )?;
                                }
                            } else if j1 > j {
                                println!("Animate move left {:?}", tile.animate_from);
                                if let None = tile.animate_from.unwrap().2 {
                                    tile.animate_from = Some((i1, j1, Some(j1 as u32 * tile_gap)));
                                    self.drawing_animation = true;
                                } else if pos > Some(j as u32 * tile_gap) && pos > Some(20) {
                                    tile.animate_from = Some((i1, j1, Some(pos.unwrap() - 20)));
                                } else {
                                    tile.animate_from = None;
                                    self.drawing_animation = false;
                                }

                                if let Some(_) = tile.animate_from {
                                    graphics::draw(
                                        ctx,
                                        &tile.image_buf,
                                        (Vec2::new(
                                            tile.animate_from.unwrap().2.unwrap() as f32 + 90.0,
                                            (i as u32 * tile_gap) as f32 + 150.0,
                                        ),),
                                    )?;
                                }
                                // Move left
                            } else if i1 > i {
                                // Move up
                                println!("Animate move up {:?}", tile.animate_from);
                                if let None = tile.animate_from.unwrap().2 {
                                    tile.animate_from = Some((i1, j1, Some(i1 as u32 * tile_gap)));
                                    self.drawing_animation = true;
                                } else if pos > Some(i as u32 * tile_gap) && pos > Some(20) {
                                    tile.animate_from = Some((i1, j1, Some(pos.unwrap() - 20)));
                                } else {
                                    tile.animate_from = None;
                                    self.drawing_animation = false;
                                }

                                if let Some(_) = tile.animate_from {
                                    graphics::draw(
                                        ctx,
                                        &tile.image_buf,
                                        (Vec2::new(
                                            (j as u32 * tile_gap) as f32 + 90.0,
                                            tile.animate_from.unwrap().2.unwrap() as f32 + 150.0,
                                        ),),
                                    )?;
                                }
                            } else if i > i1 {
                                // Move down
                                println!("Animate move down {:?}", tile.animate_from);
                                if let None = tile.animate_from.unwrap().2 {
                                    tile.animate_from = Some((i1, j1, Some(i1 as u32 * tile_gap)));
                                    self.drawing_animation = true;
                                } else if pos < Some(i as u32 * tile_gap) {
                                    tile.animate_from = Some((i1, j1, Some(pos.unwrap() + 20)));
                                } else {
                                    tile.animate_from = None;
                                    self.drawing_animation = false;
                                }

                                if let Some(_) = tile.animate_from {
                                    graphics::draw(
                                        ctx,
                                        &tile.image_buf,
                                        (Vec2::new(
                                            (j as u32 * tile_gap) as f32 + 90.0,
                                            tile.animate_from.unwrap().2.unwrap() as f32 + 150.0,
                                        ),),
                                    )?;
                                }
                            }
                        }
                    }
                } else {
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
