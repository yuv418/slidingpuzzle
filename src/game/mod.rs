use self::gmenu::game_menu::GameMenu;
use self::gmenu::main_menu::MainMenu;
use self::player::Player;
use self::scene::Scene;
use ggez::event;
use ggez::glam::Vec2;
use ggez::graphics;
use ggez::graphics::Color;
use ggez::graphics::DrawMode;
use ggez::graphics::Mesh;
use ggez::graphics::Rect;
use ggez::input::keyboard::KeyInput;
use ggez::{Context, GameResult};
use keyframe::functions::EaseInOut;
use keyframe::keyframes;
use keyframe::AnimationSequence;

pub mod animation;
pub mod drawable;
pub mod gmenu;
pub mod player;
pub mod scene;
pub mod tiles;
pub mod ui;

#[cfg(feature = "multiplayer")]
pub mod multiplayer;

pub struct GameState {
    pub current_scene: Box<dyn Scene>,
    pub prev_scene: Option<Box<dyn Scene>>,

    pub set_winsize: bool,

    // Scene transition animation variables
    scene_transition: Option<AnimationSequence<f32>>,
}

impl GameState {
    pub fn new(context: &mut Context, intro: bool) -> GameResult<Self> {
        // Loop through and make the tiles
        Ok(Self {
            current_scene: if intro {
                Box::new(player::settings_scene::SettingsScene::new(context, true)?)
            } else {
                Box::new(GameMenu::new::<MainMenu>(context)?)
            },
            prev_scene: None,
            set_winsize: false,
            scene_transition: None,
        })
    }
}

impl event::EventHandler<ggez::GameError> for GameState {
    fn text_input_event(&mut self, ctx: &mut ggez::Context, c: char) -> GameResult {
        self.current_scene.text_input_event(ctx, c);
        Ok(())
    }
    fn key_down_event(
        &mut self,
        ctx: &mut Context,
        key_input: KeyInput,
        repeat: bool,
    ) -> GameResult {
        self.current_scene.handle_key_event(ctx, key_input, repeat);
        Ok(())
    }
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        if let None = self.scene_transition {
            if let Some(next_scene) = self.current_scene.next_scene(ctx) {
                self.prev_scene = Some(std::mem::replace(&mut self.current_scene, next_scene));
                self.scene_transition = Some(keyframes![
                    (0.0, 0.0, EaseInOut),
                    (ctx.gfx.drawable_size().1, 0.2, EaseInOut),
                    (0.0, 0.4, EaseInOut)
                ]);
            }
        }
        self.current_scene.update(ctx)?;
        Ok(())
    }
    fn draw(&mut self, ctx: &mut ggez::Context) -> GameResult {
        let mut canvas =
            graphics::Canvas::from_frame(ctx, graphics::Color::from([1.0, 1.0, 1.0, 1.0]));

        if let Some(seq) = &mut self.scene_transition {
            seq.advance_by(0.01);
            let drawable_size = ctx.gfx.drawable_size();
            let cover_rect = Mesh::new_rectangle(
                ctx,
                DrawMode::fill(),
                Rect {
                    x: 0.0,
                    y: if seq.progress() > 0.5 {
                        drawable_size.1 - seq.now()
                    } else {
                        0.0
                    },
                    w: drawable_size.0,
                    h: seq.now(),
                },
                Color::WHITE,
            )?;

            if seq.progress() < 0.5 {
                self.prev_scene
                    .as_mut()
                    .unwrap()
                    .draw_transition(ctx, &mut canvas)?;
            } else if seq.progress() == 0.5 {
                self.prev_scene = None;
            } else {
                self.current_scene.draw_transition(ctx, &mut canvas)?;
            }

            canvas.draw(&cover_rect, Vec2::new(0.0, 0.0));
            if seq.finished() {
                self.scene_transition = None;
            }
        } else {
            self.current_scene.draw(ctx, &mut canvas)?;
        }
        canvas.finish(ctx)
    }
}
