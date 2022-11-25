use std::{cell::RefCell, rc::Rc};

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
    pub fn new(row_cnt_tiles: usize, col_cnt_tiles: usize) -> Self {
        Self {
            tiles: vec![],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            blank_cell: (0, 0),
            drawing_animation: false,
            game_completed: false,
        }
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
