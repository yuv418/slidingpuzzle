use std::sync::Mutex;

use ggez::{
    graphics::{Color, FontData},
    Context, GameError, GameResult,
};
use serde::{Deserialize, Serialize};

use lazy_static::lazy_static;

lazy_static! {
    static ref THEME: Mutex<Option<Theme>> = Mutex::new(None);
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Theme {
    pub bg_color: Color,
    // Also text color.
    pub fg_color: Color,
    pub border_color: Color,
    pub error_color: Color,
    pub sep_color: Color,
    // This can be loaded from /fonts/{font}.ttf
    pub font: String,
}

impl Theme {
    // NOTE that this will overwrite a theme that has already been loaded.
    pub fn load(ctx: &mut Context) -> GameResult {
        let mut theme = THEME.lock().unwrap();
        *theme = Some(
            serde_json::from_reader(ctx.fs.open("/theme.json")?)
                .map_err(|e| GameError::FilesystemError(format!("Failed to read theme: {}", e)))?,
        );

        // Load font
        let theme = theme.as_ref().unwrap();
        ctx.gfx.add_font(&theme.font, FontData::from_path(ctx, format!("/fonts/{}.ttf", theme.font))?);

        Ok(())
    }

    // TODO IMPORTANT are these functions memoized?
    pub fn bg_color() -> Color { THEME.lock().unwrap().as_ref().unwrap().bg_color }
    pub fn fg_color() -> Color { THEME.lock().unwrap().as_ref().unwrap().fg_color }
    pub fn border_color() -> Color { THEME.lock().unwrap().as_ref().unwrap().border_color }
    pub fn error_color() -> Color { THEME.lock().unwrap().as_ref().unwrap().error_color }
    pub fn sep_color() -> Color { THEME.lock().unwrap().as_ref().unwrap().sep_color }
    pub fn font() -> String { THEME.lock().unwrap().as_ref().unwrap().font.clone() }
}
