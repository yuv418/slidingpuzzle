// Display a puzzle, choose singleplayer or multiplayer
//
//
// Multiplayer host workflow
// -> Choose multiplayer
// -> Display string + copy to clipboard
// -> Start game
//
// Multiplayer client workflow
// -> Join multiplayer
// -> Paste connection string
// -> Display string + copy to clipboard
// -> Start game

use std::sync::Arc;

use ggez::{
    graphics::{DrawParam, Image},
    input::keyboard::KeyInput,
    winit::event::VirtualKeyCode,
    Context, GameResult,
};

use crate::game::{
    animation::DrawablePos,
    drawable::Drawable,
    gmenu::menu_item_list::{GameMenuItemList, NewGameMenuItemData, NewGameMenuItemDataVariant},
    input::InputAction,
    multiplayer::join_scene::JoinMultiplayerScene,
    player::PLAYER,
    puzzle::{puzzle_listing::PuzzleListing, tiles::TileState},
    resources::{image_loader::ImageLoader, theme::Theme},
    scene::Scene,
    ui::uitext::UIText,
};

pub struct PuzzleView {
    title_text: UIText,

    // Image
    puzzle_image: Arc<Image>,

    puzzle_action_mappings: GameMenuItemList,
    puzzle_num: usize,

    back: bool,
}

fn create_singleplayer_game(context: &mut Context, puzzle_num: usize) -> Box<dyn Scene> {
    let opt_player = PLAYER.lock().unwrap();
    let player = opt_player.as_ref().unwrap();
    let pos = TileState::center_xy(context);
    Box::new(
        TileState::new_singleplayer(context, puzzle_num, player.player_settings.num_rows_cols, pos)
            .expect("Failed to create singleplayer game"),
    )
}

fn create_multiplayer_game(context: &mut Context, puzzle_num: usize) -> Box<dyn Scene> {
    Box::new(JoinMultiplayerScene::new(context, puzzle_num, true).expect("Failed to create join multiplayer scene"))
}

impl PuzzleView {
    pub fn new(ctx: &mut Context, puzzle_num: usize) -> GameResult<Self> {
        let puzzle_action_mappings = GameMenuItemList::new(
            ctx,
            vec![
                NewGameMenuItemData {
                    variant: NewGameMenuItemDataVariant::TextItem { text: "Play as Singleplayer".to_string() },
                    next_page: Some(Box::new(move |c| create_singleplayer_game(c, puzzle_num))),
                },
                NewGameMenuItemData {
                    variant: NewGameMenuItemDataVariant::TextItem { text: "Create Multiplayer Game".to_string() },
                    next_page: Some(Box::new(move |c| create_multiplayer_game(c, puzzle_num))),
                },
            ],
            90.0,
            // How do we know this position for sure?
            520.0,
            500.0,
            80.0,
        )?;
        Ok(Self {
            title_text: UIText::new(format!("Puzzle {}", puzzle_num + 1), Theme::fg_color(), 78.0, DrawablePos { x: 90.0, y: 90.0 }),
            back: false,
            puzzle_num,
            puzzle_action_mappings,
            // We can panic here since the image should always be valid
            puzzle_image: ImageLoader::get_img(puzzle_num).expect("Incorrect image provided to ImageLoader"),
        })
    }
}

impl Drawable for PuzzleView {
    fn draw(&mut self, ctx: &mut Context, canvas: &mut ggez::graphics::Canvas) -> GameResult {
        let scale_factor = 300.0 / self.puzzle_image.width() as f32;
        let text_dim = self.title_text.text.measure(ctx)?;
        canvas.draw(&*self.puzzle_image, DrawParam::from([90.0, 90.0 + text_dim.y + 20.0]).scale([scale_factor; 2]));
        self.puzzle_action_mappings.draw(ctx, canvas)?;
        self.title_text.draw(ctx, canvas)?;
        Ok(())
    }
}

impl Scene for PuzzleView {
    fn handle_input_event(&mut self, ctx: &mut Context, key_input: InputAction) {
        if let InputAction::Cancel = key_input {
            self.back = true;
        }
        self.puzzle_action_mappings.handle_input_event(ctx, key_input);
    }

    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        match self.puzzle_action_mappings.next_scene(ctx) {
            Some(next_scene) => Some(next_scene),
            None if self.back => Some(Box::new(PuzzleListing::new(ctx, self.puzzle_num / 4).expect("Failed to return to puzzle listing"))),
            None => None,
        }
    }
}
