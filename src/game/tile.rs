use ggez::graphics::Image;

#[derive(Debug, Clone)]
pub struct Tile {
    // The size of a square tile (one side) in px
    pub side_len: u32,
    pub image_buf: Image,
}

#[derive(Debug)]
pub struct TileState<'a> {
    pub tiles: Vec<Vec<Tile>>,
    pub ref_board: Vec<Vec<Option<&'a Tile>>>,
}
