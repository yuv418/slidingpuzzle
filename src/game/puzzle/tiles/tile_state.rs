// TODO move animation code to tile_animation.rs

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
    animation::{
        animatable::Animatable,
        animation::{Animation, AnimationData},
    },
    drawable::Drawable,
    player::{PuzzleStatistics, PLAYER},
    puzzle::puzzle_listing::PuzzleListing,
    scene::Scene,
};

#[cfg(feature = "multiplayer")]
use crate::game::multiplayer::MultiplayerGameMessage;

use super::{tile_multiplayer::TileMultiplayerTransport, tile_random::TileRandom, Tile, TilePosition};

enum GameStage {
    StartingAnimation,
    Started,
    FinishingAnimation,
    Finished,
    Cancelled,
}

impl Default for GameStage {
    fn default() -> Self {
        Self::StartingAnimation
    }
}

// TODO: Add tile scale animation when the game is finished.

const TOTAL_SCRAMBLE_SWAPS: u32 = 50;
const TILE_SLIDE_DURATION: f32 = 0.3;

#[derive(Default)]
pub struct TileState {
    pub tiles: Vec<Vec<Rc<RefCell<Tile>>>>,
    pub ref_board: Vec<Vec<Option<Rc<RefCell<Tile>>>>>,
    // For efficiency purposes
    pub blank_cell: (usize, usize),
    pub tile_blank_cell: (usize, usize),

    previous_swap: Option<(usize, usize)>,

    animation: Animation<TilePosition>,
    img_num: usize,
    total_moves: u32,
    timer: Option<TimeContext>,

    game_stage: GameStage,

    // Multiplayer stuff
    transport: TileMultiplayerTransport,
    peer: bool,

    pub puzzle_statistics: Option<PuzzleStatistics>,
    pub x: f32,
    pub y: f32,
}

impl TileState {
    pub fn new_singleplayer(context: &mut Context, img_num: usize, num_rows_cols: usize, x: f32, y: f32) -> GameResult<Self> {
        Self::new(context, img_num, num_rows_cols, x, y, TileMultiplayerTransport::new(None), false)
    }
    pub fn new(
        context: &mut Context, img_num: usize, num_rows_cols: usize, x: f32, y: f32, transport: TileMultiplayerTransport, peer: bool,
    ) -> GameResult<Self> {
        // Peer determines whether or not a game is multiplayer
        let mut img = ImageReader::new(BufReader::new(context.fs.open(format!("/images/{}.jpg", img_num))?));
        img.set_format(image::ImageFormat::Jpeg);

        let img = img.decode().expect("failed to open image").resize(600, 600, FilterType::Lanczos3);

        // How many tiles in a row? In a column?
        let col_cnt_tiles = num_rows_cols; // img.width() / tile_size;
        let row_cnt_tiles = num_rows_cols; // img.height() / tile_size;

        let tile_size: u32 = img.width() / num_rows_cols as u32;

        // Use Default for this
        let mut tile_state = Self {
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            transport,
            peer,
            img_num,
            x,
            y,
            ..Default::default()
        };

        // Go through each row of tiles, looping through each tile in the row
        tile_state.animation.push_seq(AnimationData::Simultaneous);
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

                let tile_to_insert = Tile {
                    side_len: tile_size,
                    image_buf: Image::from_pixels(context, &row_buf_pix, ImageFormat::Rgba8UnormSrgb, tile_size, tile_size),
                    pos: TilePosition::from_ij_no_gap(i as usize - 1, j as usize - 1, tile_size, tile_state.x, tile_state.y),
                };
                let tile_to_insert = Rc::new(RefCell::new(tile_to_insert));

                tile_state.animation.push_seq(AnimationData::Generator((
                    tile_to_insert.clone(),
                    TilePosition::from_ij(i as usize - 1, j as usize - 1, tile_size, tile_state.x, tile_state.y),
                    TILE_SLIDE_DURATION * 4.0,
                )));
                tile_row.push(tile_to_insert);
            }

            tile_state.tiles.push(tile_row);
        }

        // Copy generated tile board to ref board
        for i in 0..row_cnt_tiles {
            for j in 0..col_cnt_tiles {
                let i = i as usize;
                let j = j as usize;
                tile_state.ref_board[i][j] = Some(tile_state.tiles[i][j].clone());
            }
        }

        // Remove one random tile from ref board.
        if !tile_state.peer {
            tile_state.delete_random_tile(None)?;
            for _ in 0..TOTAL_SCRAMBLE_SWAPS {
                tile_state.swap_random_tile_blank();
            }
        }

        tile_state.timer = Some(TimeContext::new());

        Ok(tile_state)
    }

    pub fn delete_random_tile(&mut self, peer_tile: Option<(usize, usize)>) -> GameResult {
        let (i, j) = if let Some(peer_tile) = peer_tile {
            peer_tile
        } else {
            let mut rng = rand::thread_rng();
            (rng.gen_range(0..self.tiles.len()) as usize, rng.gen_range(0..self.tiles.len()) as usize)
        };
        println!("deleting {:?} from ref board", (i, j));

        self.animation.push_seq(AnimationData::Unsimultaneous);
        self.animation.push_seq(AnimationData::Generator((
            self.ref_board[i][j].as_ref().unwrap().clone(),
            {
                let mut x =
                    TilePosition::from_ij_no_gap(i, j, self.ref_board[i][j].as_ref().unwrap().as_ref().borrow().side_len, self.x, self.y);
                x.scale = 0.0;
                x
            },
            TILE_SLIDE_DURATION * 4.0,
        )));
        self.ref_board[i][j] = None;
        self.tile_blank_cell = (i, j);
        self.blank_cell = (i, j);

        if !self.peer {
            self.transport.delete_random_tile(self.blank_cell)?;
        }
        Ok(())
    }

    pub fn finished(&self) -> bool {
        if let GameStage::Finished = self.game_stage {
            true
        } else {
            false
        }
    }

    pub fn swap_ref_tiles(&mut self, (i1, j1): (usize, usize), (i2, j2): (usize, usize), duration: f32) {
        if !self.peer {
            // Send to peer
            self.transport.swap_tiles((i1, j1), (i2, j2), duration);
        }

        {
            let old_tile = self.ref_board[i1][j1].clone();
            if let None = old_tile {
                self.blank_cell = (i2, j2);
            }

            self.ref_board[i1][j1] = self.ref_board[i2][j2].clone();
            self.ref_board[i2][j2] = old_tile;
        }
        // Update coordinates and keyframes

        let tile_update = (*self.ref_board[i1][j1].as_ref().unwrap()).as_ref().borrow_mut();

        let new_pos = TilePosition::from_ij(i1, j1, tile_update.side_len, self.x, self.y);
        self.animation.push_seq(AnimationData::Generator((self.ref_board[i1][j1].as_ref().unwrap().clone(), new_pos, TILE_SLIDE_DURATION)));
    }

    pub fn check_completed(&mut self) {
        for i in 0..self.ref_board.len() {
            for j in 0..self.ref_board[i].len() {
                if let Some(tile) = &self.ref_board[i][j] {
                    if !Rc::ptr_eq(&self.tiles[i][j], tile) {
                        return;
                    }
                }
            }
        }

        self.set_finishing_animation();
    }

    pub fn set_finishing_animation(&mut self) {
        // The game is completed, begin animations.
        self.animation.push_seq(AnimationData::Simultaneous);
        for i in 0..self.tiles.len() {
            for j in 0..self.tiles[i].len() {
                let tile = self.tiles[i][j].as_ref().borrow();
                let side_len = tile.side_len;
                self.animation.push_seq(AnimationData::Sequence((
                    self.tiles[i][j].clone(),
                    tile.to_state(
                        TilePosition::from_ij_no_gap(i as usize, j as usize, side_len, self.x, self.y),
                        TILE_SLIDE_DURATION * 4.0,
                    ),
                )));
            }
        }
        self.game_stage = GameStage::FinishingAnimation;
    }

    pub fn swap_random_tile_blank(&mut self) {
        // This is so low effort
        // Better method: start with gap at 0,0 and swap the gap with random adjacents over and over
        let col_cnt_tiles = self.tiles.len();
        let row_cnt_tiles = self.tiles[0].len();

        // Choose a random adjacent tile
        let tile2 = TileRandom::random_adjacent_tile(self.blank_cell, col_cnt_tiles, row_cnt_tiles);

        // Redo the method if the tile2 was the previous swap
        if Some(tile2) == self.previous_swap {
            self.swap_random_tile_blank();
            return;
        } else {
            self.previous_swap = Some(self.blank_cell);
            self.swap_ref_tiles(self.blank_cell, tile2, TILE_SLIDE_DURATION)
        }
    }

    pub fn get_puzzle_statistics(&self) -> PuzzleStatistics {
        PuzzleStatistics {
            finish_time: Local::now(),
            duration: self.timer.as_ref().unwrap().time_since_start(),
            move_count: self.total_moves,
        }
    }
}
impl Scene for TileState {
    #[cfg(feature = "multiplayer")]
    fn update(&mut self, _ctx: &mut Context) -> GameResult {
        use log::trace;

        if self.peer {
            if self.animation.finished() {
                if let Some(msg) = self.transport.recv_message() {
                    trace!("recv tile msg {:?}", msg);
                    match msg {
                        MultiplayerGameMessage::SwapTiles { i1j1, i2j2, duration } => {
                            self.swap_ref_tiles(i1j1, i2j2, duration);
                        }
                        MultiplayerGameMessage::DeleteRandomTile((i, j)) => {
                            // TODO move this to separate function to deal with animations
                            self.delete_random_tile(Some((i, j)))?;
                        }
                        MultiplayerGameMessage::GameCompleted(stats) => {
                            // TODO move this to separate function to deal with animations
                            self.puzzle_statistics = Some(stats);
                            println!("Peer completed game");
                            self.set_finishing_animation();
                        }
                        MultiplayerGameMessage::ScramblingFinished => {
                            self.game_stage = GameStage::Started;
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, _ctx: &mut Context, key_input: KeyInput, repeat: bool) {
        let (i, j) = self.blank_cell;
        let mut swap_tile = (i, j);

        // TODO how do we make escape callable during animation?
        if !repeat {
            if let GameStage::Started = self.game_stage {
                if let Some(vkeycode) = key_input.keycode {
                    match vkeycode {
                        // Tile below space
                        VirtualKeyCode::Up if i + 1 < self.ref_board.len() => swap_tile = (i + 1, j),
                        // Tile above space
                        VirtualKeyCode::Down if i != 0 => swap_tile = (i - 1, j),
                        // Tile left of space
                        VirtualKeyCode::Left if j + 1 < self.ref_board[i].len() => swap_tile = (i, j + 1),
                        // Tile right of space
                        VirtualKeyCode::Right if j != 0 => swap_tile = (i, j - 1),
                        // Cancel game
                        VirtualKeyCode::Escape => self.game_stage = GameStage::Cancelled,
                        _ => {}
                    }
                    if swap_tile != self.blank_cell {
                        self.swap_ref_tiles(self.blank_cell, swap_tile, TILE_SLIDE_DURATION);
                        self.total_moves += 1;
                    }
                }
                // TODO move this to the update method
                if !self.peer {
                    // Immediately will happen during this
                    self.check_completed();
                    if let GameStage::FinishingAnimation = self.game_stage {
                        let stats = self.get_puzzle_statistics();
                        self.puzzle_statistics = Some(stats.clone());
                        self.transport.end_game(stats);
                    }
                }
            }
        }
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        match self.game_stage {
            GameStage::Finished | GameStage::Cancelled => {
                if let GameStage::Finished = self.game_stage {
                    // Update player completed puzzles,
                    // point value?
                    {
                        let mut opt_player = PLAYER.lock().unwrap();
                        let player = opt_player.as_mut().unwrap();

                        let game_stat = self.get_puzzle_statistics();

                        // TODO do we really want this? Should multiplayer stats get saved separately?
                        if let Some(statistics) = player.completed_puzzles.get_mut(&self.img_num) {
                            statistics.push(game_stat);
                        } else {
                            player.completed_puzzles.insert(self.img_num, vec![game_stat]);
                        }
                        player.save(ctx).expect("Failed to save player statistics");
                    }
                }
                Some(Box::new(PuzzleListing::new(ctx, 4 * ((self.img_num) / 4)).expect("Failed to return to puzzle listing")))
            }
            _ => None,
        }
    }

    fn draw_transition(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        for tr in self.tiles.iter() {
            for tile in tr {
                tile.as_ref().borrow_mut().draw(ctx, canvas)?;
            }
        }
        Ok(())
    }
}

impl Drawable for TileState {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        // draw all tiles with a 10px gap between each title
        self.animation.advance(0.05);
        if self.animation.finished() {
            match self.game_stage {
                GameStage::StartingAnimation => self.game_stage = GameStage::Started,
                GameStage::FinishingAnimation => self.game_stage = GameStage::Finished,
                _ => {}
            }
        }

        for i in 0..self.ref_board.len() {
            // each tile in the row, so x
            for j in 0..self.ref_board[i].len() {
                let mut tile = self.tiles[i][j].as_ref().borrow_mut();
                tile.draw(ctx, canvas)?;
            }
        }

        Ok(())
    }
}
