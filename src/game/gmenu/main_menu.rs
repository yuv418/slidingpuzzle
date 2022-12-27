use ggez::Context;

use crate::game::{
    multiplayer::join_scene::JoinMultiplayerScene,
    player::{settings_scene::SettingsScene, PLAYER},
    puzzle::{puzzle_listing::PuzzleListing, tiles::TileState},
    scene::Scene,
};

use super::{
    game_menu::GameMenuData,
    menu_item_list::{NewGameMenuItemData, NewGameMenuItemDataVariant},
};

pub fn continue_game(context: &mut Context) -> Box<dyn Scene> {
    let opt_player = PLAYER.lock().unwrap();
    // Player guaranteed to be some at this point
    let player = opt_player.as_ref().unwrap();

    let tile_state = Box::new(
        TileState::new_singleplayer(
            context,
            if !player.completed_puzzles.is_empty() {
                // We'll have to add some kind of check to make sure
                // that the player hasn't actually completed the entire game,
                // otherwise this would cause problems.
                player
                    .completed_puzzles
                    .iter()
                    .next_back()
                    .expect("Failed to get max completed puzzle")
                    .0
                    + 1
            } else {
                0
            },
            player.player_settings.num_rows_cols,
            0.0,
            0.0,
        )
        .expect("Failed to create TileState"),
    );

    tile_state
}

pub fn join_multiplayer(context: &mut Context) -> Box<dyn Scene> {
    Box::new(
        JoinMultiplayerScene::new(context, 0, false)
            .expect("Failed to create join multiplayer scene"),
    )
}

pub fn settings_scene(context: &mut Context) -> Box<dyn Scene> {
    Box::new(SettingsScene::new(context, false).expect("Failed to create settings scene"))
}

pub fn choose_puzzle(context: &mut Context) -> Box<dyn Scene> {
    Box::new(PuzzleListing::new(context, 0).expect("Failed to create puzzle listing"))
}

pub struct MainMenu {}

impl GameMenuData for MainMenu {
    fn menu_mappings() -> Vec<NewGameMenuItemData> {
        vec![
            NewGameMenuItemData {
                variant: NewGameMenuItemDataVariant::TextItem { text: "Continue".to_string() },
                next_page: Some(Box::new(continue_game)),
            },
            NewGameMenuItemData {
                variant: NewGameMenuItemDataVariant::TextItem {
                    text: "Join Multiplayer".to_string(),
                },
                next_page: Some(Box::new(join_multiplayer)),
            },
            NewGameMenuItemData {
                variant: NewGameMenuItemDataVariant::TextItem {
                    text: "Choose a Puzzle".to_string(),
                },
                next_page: Some(Box::new(choose_puzzle)),
            },
            NewGameMenuItemData {
                variant: NewGameMenuItemDataVariant::TextItem { text: "Settings".to_string() },
                next_page: Some(Box::new(settings_scene)),
            },
        ]
    }

    fn title() -> String {
        "Sliding Puzzle".to_string()
    }
}
