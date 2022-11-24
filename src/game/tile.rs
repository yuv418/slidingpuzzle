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
}

impl TileState {
    pub fn swap_ref_tiles(&mut self, (i1, j1): (usize, usize), (i2, j2): (usize, usize)) {
        let old_tile = self.ref_board[i1][j1].clone();
        if let None = old_tile {
            self.blank_cell = (i2, j2);
        }

        self.ref_board[i1][j1] = self.ref_board[i2][j2].clone();
        self.ref_board[i2][j2] = old_tile;
    }
    pub fn check_completed(&self) -> bool {
        false
    }
}
