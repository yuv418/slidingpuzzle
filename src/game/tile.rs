use image::io::Reader as ImageReader;
use image::GenericImageView;
use image::Pixel;
use keyframe::functions::EaseInOut;
use keyframe::keyframes;
use keyframe::AnimationSequence;
use keyframe_derive::CanTween;
use rand::Rng;
use std::borrow::BorrowMut;
use std::{cell::RefCell, path::PathBuf, rc::Rc};

use ggez::{
    graphics::{self, Image},
    Context, GameResult,
};
use glam::Vec2;

use crate::game::drawable::Drawable;

const TILE_GAP: f32 = 20.0;

const TILE_PADDING_X: f32 = 90.0;
const TILE_PADDING_Y: f32 = 150.0;

pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
    pub pos: TilePosition,
    pub animation: Option<AnimationSequence<TilePosition>>,
}

#[derive(CanTween, Debug, Clone, Copy, Default)]
pub struct TilePosition {
    pub x: f32,
    pub y: f32,
}

impl TilePosition {
    pub fn from_ij(i: usize, j: usize, tile_size: u32) -> Self {
        TilePosition {
            x: TILE_PADDING_X + (j as f32 * (tile_size as f32 + TILE_GAP)),
            y: TILE_PADDING_Y + (i as f32 * (tile_size as f32 + TILE_GAP)),
        }
    }
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
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        if let Some(seq) = &mut self.animation {
            seq.advance_by(0.05);
            let anim_pos = seq.now();
            graphics::draw(ctx, &self.image_buf, (Vec2::new(anim_pos.x, anim_pos.y),))?;
            if seq.finished() {
                self.animation = None;
            }
        } else {
            graphics::draw(ctx, &self.image_buf, (Vec2::new(self.pos.x, self.pos.y),))?;
        }
        Ok(())
    }
}

pub struct TileState {
    pub tiles: Vec<Vec<Rc<RefCell<Tile>>>>,
    pub ref_board: Vec<Vec<Option<Rc<RefCell<Tile>>>>>,
    // For efficiency purposes
    pub blank_cell: (usize, usize),
    pub tile_blank_cell: (usize, usize),
    pub drawing_animation: bool,
    game_completed: bool,
    pub x: f32,
    pub y: f32,
}

impl TileState {
    pub fn new(
        context: &mut Context,
        img_path: PathBuf,
        tile_size: u32,
        x: f32,
        y: f32,
    ) -> GameResult<Self> {
        let img = ImageReader::open(img_path)?
            .decode()
            .expect("failed to open image");

        // How many tiles in a row? In a column?
        let col_cnt_tiles = img.width() / tile_size;
        let row_cnt_tiles = img.height() / tile_size;

        let mut tile_state = Self {
            tiles: vec![],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            blank_cell: (0, 0),
            tile_blank_cell: (0, 0),
            drawing_animation: false,
            game_completed: false,
            x,
            y,
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
                        context,
                        tile_size as u16,
                        tile_size as u16,
                        &row_buf_pix,
                    )?,
                    pos: TilePosition::from_ij(i as usize, j as usize, tile_size),
                    animation: None,
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
        tile_state.tile_blank_cell = (i, j);
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
        // Fix the missing tile
        let (m_x, m_y) = tile_state.tile_blank_cell;
        tile_state.tiles[m_x][m_y].as_ref().borrow_mut().pos =
            TilePosition::from_ij(m_x, m_y, tile_size);

        Ok(tile_state)
    }
    pub fn swap_ref_tiles(
        &mut self,
        (i1, j1): (usize, usize),
        (i2, j2): (usize, usize),
        animate: bool,
    ) {
        {
            let old_tile = self.ref_board[i1][j1].clone();
            if let None = old_tile {
                self.blank_cell = (i2, j2);
            }

            self.ref_board[i1][j1] = self.ref_board[i2][j2].clone();
            self.ref_board[i2][j2] = old_tile;
        }
        // Update coordinates and keyframes

        let mut tile_update = (*self.ref_board[i1][j1].as_ref().unwrap())
            .as_ref()
            .borrow_mut();

        let new_pos = TilePosition::from_ij(i1, j1, tile_update.side_len);
        if animate {
            tile_update.animation = Some(keyframes![
                (tile_update.pos.clone(), 0.0, EaseInOut),
                (new_pos.clone(), 0.2, EaseInOut)
            ]);
        }
        tile_update.pos = new_pos;
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
    fn draw(&mut self, ctx: &mut Context) -> GameResult {
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
                .as_ref()
                .borrow_mut();

                tile.draw(ctx)?;
            }
        }
        graphics::present(ctx)?;

        Ok(())
    }
}
