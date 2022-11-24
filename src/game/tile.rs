use std::rc::Rc;

use ggez::graphics::Image;

#[derive(Debug, Clone)]
pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
}

#[derive(Debug)]
pub struct TileState {
    pub tiles: Vec<Vec<Rc<Tile>>>,
    pub ref_board: Vec<Vec<Option<Rc<Tile>>>>,
    // For efficiency purposes
    pub blank_cell: (usize, usize),
    game_completed: bool,
}

impl TileState {
    pub fn new(row_cnt_tiles: usize, col_cnt_tiles: usize) -> Self {
        Self {
            tiles: vec![],
            ref_board: vec![vec![None; col_cnt_tiles as usize]; row_cnt_tiles as usize],
            blank_cell: (0, 0),
            game_completed: false,
        }
    }
    pub fn swap_ref_tiles(&mut self, (i1, j1): (usize, usize), (i2, j2): (usize, usize)) {
        let old_tile = self.ref_board[i1][j1].clone();
        if let None = old_tile {
            self.blank_cell = (i2, j2);
        }

        self.ref_board[i1][j1] = self.ref_board[i2][j2].clone();
        self.ref_board[i2][j2] = old_tile;
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
