use ggez::{Context, GameResult};

use crate::game::{
    animation::DrawablePos,
    drawable::Drawable,
    gmenu::{game_menu::GameMenu, main_menu::MainMenu, menu_item::GameMenuItem},
    input::InputAction,
    player::PLAYER,
    puzzle::puzzle_view::PuzzleView,
    resources::{image_loader::ImageLoader, theme::Theme},
    scene::Scene,
    ui::uitext::UIText,
};

#[derive(Debug)]
enum PaginationDirection {
    Left,
    Right,
}

// Listing is 2x2
pub struct PuzzleListing {
    listing_start: usize,
    currently_selected: (usize, usize),
    menu_items: [[Option<GameMenuItem>; 2]; 2],
    title_mesh: UIText,
    page_direction: Option<PaginationDirection>,
    back: bool,
    start_game: bool,
}

impl PuzzleListing {
    pub fn new(ctx: &mut Context, listing_start: usize) -> GameResult<Self> {
        let title_mesh = UIText::new(
            format!("Puzzles {} to {}", listing_start + 1, listing_start + 4),
            Theme::fg_color(),
            78.0,
            DrawablePos { x: 45.0, y: 45.0 },
        );

        let t_sz = title_mesh.text.measure(ctx)?;

        let mut menu_items: [[Option<GameMenuItem>; 2]; 2] = [[None, None], [None, None]];

        for i in 0..menu_items.len() {
            for j in 0..menu_items[i].len() {
                let puzzle_num = listing_start + (i * 2) + j;
                if let Some(img) = ImageLoader::get_img(puzzle_num) {
                    menu_items[i][j] = Some(GameMenuItem::new_image_item(
                        ctx,
                        img,
                        &format!("Puzzle {}", puzzle_num + 1),
                        // This should never happen, so we can panic if it does.
                        Some(Box::new(|_| -> Box<dyn Scene> { panic!() })),
                        45.0 + (j as f32 * 320.0),
                        55.0 + t_sz.y + (i as f32 * 320.0),
                        300.0,
                        300.0,
                    )?);
                }
            }
        }

        // TODO guarantee that at least one item is present
        menu_items[0][0].as_mut().unwrap().select();

        Ok(Self { currently_selected: (0, 0), listing_start, title_mesh, menu_items, page_direction: None, back: false, start_game: false })
    }
}

impl Drawable for PuzzleListing {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        self.title_mesh.draw(ctx, canvas)?;

        for listing_row in &mut self.menu_items {
            for listing in listing_row {
                if let Some(listing) = listing {
                    listing.draw(ctx, canvas)?;
                }
            }
        }
        Ok(())
    }
}

impl Scene for PuzzleListing {
    fn handle_input_event(&mut self, _ctx: &mut ggez::Context, key_input: InputAction) {
        let old_selected = self.currently_selected;

        let (o_i, o_j) = old_selected;

        match key_input {
            InputAction::Up =>
                if self.currently_selected.0 > 0 {
                    self.currently_selected.0 -= 1;
                },

            InputAction::Down =>
                if self.currently_selected.0 < 1 && self.menu_items[o_i + 1][o_j].is_some() {
                    self.currently_selected.0 += 1;
                },
            InputAction::Left => {
                if self.currently_selected.1 > 0 {
                    self.currently_selected.1 -= 1;
                } else {
                    // Page left
                    self.page_direction = Some(PaginationDirection::Left);
                }
            }
            InputAction::Right => {
                if self.currently_selected.1 < 1 && self.menu_items[o_i][o_j + 1].is_some() {
                    self.currently_selected.1 += 1;
                } else {
                    // Page right
                    self.page_direction = Some(PaginationDirection::Right);
                }
            }
            InputAction::Cancel => {
                self.back = true;
            }
            InputAction::Select => {
                self.start_game = true;
            }
            _ => {}
        }
        if old_selected != self.currently_selected {
            self.menu_items[old_selected.0][old_selected.1].as_mut().unwrap().deselect();
            self.menu_items[self.currently_selected.0][self.currently_selected.1].as_mut().unwrap().select();
        }
    }

    fn next_scene(&mut self, ctx: &mut ggez::Context) -> Option<Box<dyn Scene>> {
        if self.back {
            return Some(Box::new(GameMenu::new::<MainMenu>(ctx).expect("Failed to launch game menu")));
        } else if self.start_game {
            let opt_player = PLAYER.lock().unwrap();
            // Player guaranteed to be some at this point
            let _player = opt_player.as_ref().unwrap();

            let game_image_num = self.listing_start + (self.currently_selected.0 * 2) + self.currently_selected.1;

            println!("starting tile state {}", game_image_num);
            return Some(
                // TODO move this the puzzle view
                Box::new(PuzzleView::new(ctx, game_image_num).expect("Failed to create tile state")),
            );
        }

        let check_listing = match self.page_direction {
            None => return None,
            Some(PaginationDirection::Left) =>
                if self.listing_start >= 4 {
                    self.listing_start - 4
                } else {
                    return None;
                },

            Some(PaginationDirection::Right) => self.listing_start + 4,
        };

        if ImageLoader::has_img(check_listing) {
            Some(Box::new(Self::new(ctx, check_listing).expect("Failed to make prev page of puzzles")))
        } else {
            self.page_direction = None;
            None
        }
    }

    fn draw_transition(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        self.draw(ctx, canvas)
    }

    fn text_input_event(&mut self, _ctx: &mut ggez::Context, _c: char) {}
}
