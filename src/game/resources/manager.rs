use crate::game::{
    drawable::Drawable,
    gmenu::{game_menu::GameMenu, main_menu::MainMenu},
    player::{settings_scene::SettingsScene, Player},
    scene::Scene,
};
use ggez::{
    graphics::{Canvas, DrawParam, Text, TextFragment},
    Context, GameResult,
};

use super::{image_loader::ImageLoader, theme::Theme};

#[derive(Default)]
pub struct ResourceManager {
    intro: bool,
    player_loaded: bool,
    theme_loaded: bool,
    images_loaded: bool,
}
impl ResourceManager {
    pub fn new(ctx: &mut Context) -> GameResult<Self> {
        // So the main UI doesn't panic
        Theme::load(ctx)?;
        Ok(Self { theme_loaded: true, ..Default::default() })
    }
}

impl Scene for ResourceManager {
    fn next_scene(&mut self, ctx: &mut Context) -> Option<Box<dyn Scene>> {
        if self.player_loaded && self.theme_loaded && self.images_loaded {
            // TODO better error handling here.
            return Some(if self.intro {
                Box::new(SettingsScene::new(ctx, true).ok()?)
            } else {
                Box::new(GameMenu::new::<MainMenu>(ctx).ok()?)
            });
        }
        None
    }
}

impl Drawable for ResourceManager {
    #[rustfmt::skip]
    fn draw(&mut self, ctx: &mut Context, canvas: &mut Canvas) -> GameResult {
        let mut status_text = "Sliding Puzzle Resource Loader\n\n".to_string();

        if !self.player_loaded { self.intro = Player::startup(ctx); self.player_loaded = true; }

        if self.player_loaded { status_text += "Player loaded successfully.\n" }
        if self.theme_loaded { status_text += "Theme loaded successfully.\n" }
        if !self.images_loaded {
            let (loaded, total) = ImageLoader::get_load_status(ctx);
            status_text += &format!("{}/{} images loaded.\n", loaded, total);
            if loaded / total == 1 {
                self.images_loaded = true;
            }
        }


        // No resources are loaded, so we cannot use UIText.
        let text = Text::new(TextFragment { text: status_text, color: Some(Theme::fg_color()), ..Default::default() });
        canvas.draw(&text, DrawParam::from([20.0, 20.0]));

        Ok(())
    }
}
