use chrono::Local;
use image::{imageops::FilterType, io::Reader as ImageReader, GenericImageView, Pixel};
use rand::Rng;
use std::{cell::RefCell, io::BufReader, rc::Rc};

use ggez::{
    graphics::{Canvas, Image, ImageFormat},
    input::keyboard::KeyInput,
    timer::TimeContext,
    winit::event::VirtualKeyCode,
    Context, GameResult,
};

use crate::game::{
    drawable::Drawable,
    gmenu::puzzle_listing::PuzzleListing,
    player::{Player, PuzzleStatistics, PLAYER},
    scene::Scene,
};

use super::{Tile, TilePosition};

const TOTAL_SCRAMBLE_SWAPS: u32 = 50;
const TILE_SLIDE_DURATION: f32 = 0.2;

pub struct TileState {
    pub tiles: Vec<Vec<Rc<RefCell<Tile>>>>,
    pub ref_board: Vec<Vec<Option<Rc<RefCell<Tile>>>>>,
    // For efficiency purposes
    pub blank_cell: (usize, usize),
    pub tile_blank_cell: (usize, usize),
    // For the initial outwards animation and shuffling
    outwards_animated_tiles: usize,
    total_tiles_swapped: u32,
    game_started: bool,

    // For when the game is finished
    inwards_animated_tiles: usize,
    game_completed: bool,

    current_animation: Option<(usize, usize)>,
    // Previous from redoing the previous swap
    previous_swap: Option<(usize, usize)>,

    swapping_tiles: bool,

    img_num: usize,
    total_moves: u32,
    timer: Option<TimeContext>,

    pub x: f32,
    pub y: f32,
}

impl TileState {
    pub fn new(context: &mut Context, img_num: usize, num_rows_cols: usize) -> GameResult<Self> {
        let mut img = ImageReader::new(BufReader::new(
            context.fs.open(format!("/images/{}.jpg", img_num))?,
        ));
        img.set_format(image::ImageFormat::Jpeg);

        let img =
            img.decode()
                .expect("failed to open image")
                .resize(600, 600, FilterType::Lanczos3);

        // How many tiles in a row? In a column?
        let col_cnt_tiles = num_rows_cols; // img.width() / tile_size;
        let row_cnt_tiles = num_rows_cols; // img.height() / tile_size;

        let tile_size: u32 = img.width() / num_rows_cols as u32;

        let mut tile_state = Self {
            tiles: vec![],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            blank_cell: (0, 0),
            tile_blank_cell: (0, 0),
            game_started: false,
            outwards_animated_tiles: 0,
            total_tiles_swapped: 0,
            inwards_animated_tiles: 0,
            swapping_tiles: false,
            game_completed: false,
            current_animation: None,
            previous_swap: None,
            img_num,
            total_moves: 0,
            timer: None,
            x: 0.0,
            y: 0.0,
        };

        // Go through each row of tiles, looping through each tile in the row
        for i in 1..(row_cnt_tiles + 1) {
            let mut tile_row = vec![];
            for j in 1..(col_cnt_tiles + 1) {
                // Go through each square block, adding a tile to our TileState
                let mut row_buf_pix = vec![];

                let i = i as u32;
                let j = j as u32;

                for y in (tile_size * (i - 1))..(tile_size * i) {
                    // add each row individiaully to the tile_to_insert
                    // row buffer of pixels
                    for x in (tile_size * (j - 1))..(tile_size * j) {
                        let pix_rgba = img.get_pixel(x, y).to_rgba().0;
                        for rgba_val in pix_rgba {
                            row_buf_pix.push(rgba_val);
                        }
                    }
                    // println!("one-row length is {:?}", row_buf_pix.len());
                }
                println!("writing tile to tile_row");
                let tile_to_insert = Tile {
                    side_len: tile_size,
                    image_buf: Image::from_pixels(
                        context,
                        &row_buf_pix,
                        ImageFormat::Rgba8UnormSrgb,
                        tile_size,
                        tile_size,
                    ),
                    pos: TilePosition::from_ij_no_gap(i as usize - 1, j as usize - 1, tile_size),
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
        println!("deleting {:?} from ref board", (i, j));
        tile_state.ref_board[i][j] = None;
        tile_state.tile_blank_cell = (i, j);
        tile_state.blank_cell = (i, j);

        let tile_gap = tile_state.tiles[0][0].borrow().side_len + 10; // determine the gap here
        let win_width = (180 + (tile_state.tiles[0].len() as u32 * tile_gap)) as f32;
        let win_height = (300 + (tile_gap * tile_state.tiles.len() as u32)) as f32;
        println!("the new window dimensions are {}x{}", win_width, win_height);
        // This should happen in TileState::new, not here.
        context
            .gfx
            .set_mode(
                ggez::conf::WindowMode::default()
                    .dimensions(win_width, win_height)
                    .resizable(true),
            )
            .expect("Failed to resize window for tile game");

        tile_state.timer = Some(TimeContext::new());

        Ok(tile_state)
    }
    pub fn swap_ref_tiles(
        &mut self,
        (i1, j1): (usize, usize),
        (i2, j2): (usize, usize),
        duration: f32,
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
        tile_update.to_pos(new_pos, duration);

        self.current_animation = Some((i1, j1));
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

    pub fn swap_random_tile_blank(&mut self) {
        // This is so low effort
        // Better method: start with gap at 0,0 and swap the gap with random adjacents over and over
        let mut rng = rand::thread_rng();
        let col_cnt_tiles = self.tiles.len();
        let row_cnt_tiles = self.tiles[0].len();
        // Choose a random adjacent tile
        let replacetile = if self.blank_cell == (0, 0) {
            rng.gen_range(0..2) as usize
        } else if self.blank_cell == (0, col_cnt_tiles as usize - 1) {
            let r = rng.gen_range(0..2);
            (if r == 1 { 2 } else { r } as usize)
        } else if self.blank_cell == (row_cnt_tiles as usize - 1, col_cnt_tiles as usize - 1) {
            let r = rng.gen_range(0..2);
            (if r == 0 { 3 } else { 2 } as usize)
        } else if self.blank_cell == (row_cnt_tiles as usize - 1, 0) {
            let r = rng.gen_range(0..2);
            (if r == 0 { 3 } else { r } as usize)
        }
        // Left edge
        else if self.blank_cell.1 == 0 {
            let r = rng.gen_range(0..3);
            (if r == 2 { 3 } else { r } as usize)
        }
        // Top edge
        else if self.blank_cell.0 == 0 {
            rng.gen_range(0..3)
        }
        // Right edge
        else if self.blank_cell.1 == col_cnt_tiles as usize - 1 {
            let r = rng.gen_range(0..3);
            (if r == 1 { 3 } else { r } as usize)
        }
        // Bottom edge
        else if self.blank_cell.0 == row_cnt_tiles as usize - 1 {
            rng.gen_range(0..3) + 1
        } else {
            rng.gen_range(0..4) as usize
        };

        let (c1, c2) = self.blank_cell;

        // println!("swap {:?} {:?}", self.blank_cell, replacetile);
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

        // Redo the method if the tile2 was the previous swap
        if Some(tile2) == self.previous_swap {
            self.swap_random_tile_blank();
            return;
        } else {
            self.previous_swap = Some(self.blank_cell);
            self.swap_ref_tiles(self.blank_cell, tile2, 0.15)
        }
    }
}
impl Scene for TileState {
    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, repeat: bool) {
        let i = self.blank_cell.0;
        let j = self.blank_cell.1;
        if !repeat && !self.game_completed() {
            // TODO make this DRYer
            if let Some(vkeycode) = key_input.keycode {
                match vkeycode {
                    VirtualKeyCode::Up => {
                        // Tile below space
                        if i + 1 < self.ref_board.len() {
                            self.swap_ref_tiles((i, j), (i + 1, j), TILE_SLIDE_DURATION);
                            self.total_moves += 1;
                        }
                    }
                    VirtualKeyCode::Down => {
                        // Tile above space
                        if i != 0 {
                            self.swap_ref_tiles((i, j), (i - 1, j), TILE_SLIDE_DURATION);
                            self.total_moves += 1;
                        }
                    }
                    VirtualKeyCode::Left => {
                        // Tile left of space
                        if j + 1 < self.ref_board[i].len() {
                            self.swap_ref_tiles((i, j), (i, j + 1), TILE_SLIDE_DURATION);
                            self.total_moves += 1;
                        }
                    }
                    VirtualKeyCode::Right => {
                        // Tile right of space
                        if j != 0 {
                            self.swap_ref_tiles((i, j), (i, j - 1), TILE_SLIDE_DURATION);
                            self.total_moves += 1;
                        }
                    }
                    _ => {}
                }
            }
            // TODO move this to the update method
            self.check_completed();
        }
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.game_completed() && self.current_animation.is_none() {
            // Update player completed puzzles,
            // point value?

            {
                let mut opt_player = PLAYER.lock().unwrap();
                let player = opt_player.as_mut().unwrap();

                let game_stat = PuzzleStatistics {
                    finish_time: Local::now(),
                    duration: self.timer.as_ref().unwrap().time_since_start(),
                    move_count: self.total_moves,
                };

                if let Some(statistics) = player.completed_puzzles.get_mut(&self.img_num) {
                    statistics.push(game_stat);
                } else {
                    player
                        .completed_puzzles
                        .insert(self.img_num, vec![game_stat]);
                }
                player.save(ctx).expect("Failed to save player statistics");
            }

            Some(Box::new(
                PuzzleListing::new(ctx, 4 * (self.img_num % 4))
                    .expect("Failed to return to puzzle listing"),
            ))
        } else {
            None
        }
    }

    fn draw_transition(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        for i in 0..self.ref_board.len() {
            for j in 0..self.ref_board[i].len() {
                let mut tile = self.tiles[i][j].borrow_mut();
                tile.draw(ctx, canvas)?;
            }
        }
        Ok(())
    }
}

impl Drawable for TileState {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        // draw all tiles with a 10px gap between each title

        if let Some(current_animation) = self.current_animation {
            let (a_x, a_y) = current_animation;
            if self.ref_board[a_x][a_y]
                .as_ref()
                .unwrap()
                .as_ref()
                .borrow()
                .animation
                .is_none()
            {
                self.current_animation = None;
            }
        }

        let total_tiles = self.ref_board.len() * self.ref_board[0].len();

        for i in 0..self.ref_board.len() {
            // each tile in the row, so x
            for j in 0..self.ref_board[i].len() {
                let mut tile = if !self.game_started {
                    if self.outwards_animated_tiles < total_tiles {
                        let tile = &self.tiles[i][j];
                        let mut tile_update = tile.as_ref().borrow_mut();
                        let side_len = tile_update.side_len;
                        tile_update.to_pos(TilePosition::from_ij(i, j, side_len), 3.0);
                        self.outwards_animated_tiles += 1;
                        &self.tiles[i][j]
                    } else if self.outwards_animated_tiles == total_tiles && !self.swapping_tiles {
                        self.swapping_tiles = true;
                        // This seems rather inefficient
                        for tile_row in &self.tiles {
                            for tile in tile_row {
                                if tile.borrow().animation.is_some() {
                                    self.swapping_tiles = false;
                                }
                            }
                        }
                        &self.tiles[i][j]
                    } else if self.swapping_tiles {
                        // Slide a random tile
                        if self.total_tiles_swapped < TOTAL_SCRAMBLE_SWAPS
                            && self.current_animation.is_none()
                        {
                            self.swap_random_tile_blank();
                            self.total_tiles_swapped += 1;
                        } else if self.total_tiles_swapped == TOTAL_SCRAMBLE_SWAPS {
                            // Fix the missing tile
                            let tile = &self.tiles[i][j];
                            let side_len = { tile.as_ref().borrow().side_len };
                            let (m_x, m_y) = self.tile_blank_cell;
                            self.tiles[m_x][m_y].as_ref().borrow_mut().pos =
                                TilePosition::from_ij(m_x, m_y, side_len);
                            self.game_started = true;
                        }
                        if let Some(tile) = &self.ref_board[i][j] {
                            tile
                        } else {
                            continue;
                        }
                    } else {
                        panic!("Never happens")
                    }
                } else if !self.game_completed() {
                    if let Some(tile) = &self.ref_board[i][j] {
                        tile
                    } else {
                        continue;
                    }
                } else {
                    let tile = &self.tiles[i][j];

                    if self.inwards_animated_tiles < total_tiles {
                        let mut tile_update = tile.as_ref().borrow_mut();
                        let side_len = tile_update.side_len;
                        tile_update.to_pos(TilePosition::from_ij_no_gap(i, j, side_len), 1.8);
                        self.inwards_animated_tiles += 1;
                    }
                    tile
                }
                .as_ref()
                .borrow_mut();

                tile.draw(ctx, canvas)?;
            }
        }

        Ok(())
    }
}
