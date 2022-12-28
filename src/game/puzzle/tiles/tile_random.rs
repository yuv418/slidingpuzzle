use rand::Rng;

pub struct TileRandom {}
impl TileRandom {
    pub fn random_adjacent_tile(blank_cell: (usize, usize), col_cnt_tiles: usize, row_cnt_tiles: usize) -> (usize, usize) {
        // TODO make this static, or something
        let mut rng = rand::thread_rng();

        let replacetile = if blank_cell == (0, 0) {
            rng.gen_range(0..2) as usize
        } else if blank_cell == (0, col_cnt_tiles as usize - 1) {
            let r = rng.gen_range(0..2);
            (if r == 1 { 2 } else { r } as usize)
        } else if blank_cell == (row_cnt_tiles as usize - 1, col_cnt_tiles as usize - 1) {
            let r = rng.gen_range(0..2);
            (if r == 0 { 3 } else { 2 } as usize)
        } else if blank_cell == (row_cnt_tiles as usize - 1, 0) {
            let r = rng.gen_range(0..2);
            (if r == 0 { 3 } else { r } as usize)
        }
        // Left edge
        else if blank_cell.1 == 0 {
            let r = rng.gen_range(0..3);
            (if r == 2 { 3 } else { r } as usize)
        }
        // Top edge
        else if blank_cell.0 == 0 {
            rng.gen_range(0..3)
        }
        // Right edge
        else if blank_cell.1 == col_cnt_tiles as usize - 1 {
            let r = rng.gen_range(0..3);
            (if r == 1 { 3 } else { r } as usize)
        }
        // Bottom edge
        else if blank_cell.0 == row_cnt_tiles as usize - 1 {
            rng.gen_range(0..3) + 1
        } else {
            rng.gen_range(0..4) as usize
        };

        let (c1, c2) = blank_cell;

        // println!("swap {:?} {:?}", blank_cell, replacetile);
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
        tile2
    }
}
