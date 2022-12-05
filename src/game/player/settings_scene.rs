use ggez::{
    glam::Vec2,
    graphics::{Color, PxScale, Text, TextFragment},
    input::keyboard::KeyCode,
    Context, GameResult,
};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{
    drawable::Drawable,
    gmenu::{menu_item::GameMenuItem, GameMenu},
    scene::Scene,
};

use super::{Player, PlayerSettings, PLAYER};

pub struct SettingsScene {
    intro: bool,
    intro_animation: AnimationSequence<f32>,
    greeting: Text,
    greeting_visible: bool,
    welcome: Text,
    welcome_visible: bool,
    enter_confirm: Text,
    enter_confirm_visible: bool,

    menu_visible: bool,

    // Menu options
    options: Vec<GameMenuItem>,
    selected_option: usize,

    advance_scene: bool,

    main: Text,
}

const INPUT_BOX_HEIGHT: f32 = 110.0;

impl SettingsScene {
    pub fn save_configuration(&mut self, ctx: &mut Context) -> GameResult {
        // Should be safe to unwrap here due to prior parsing
        let username = self.options[0].get_input_value().unwrap();
        let num_rows_cols = self.options[1].get_input_value().unwrap().parse().unwrap();

        let mut opt_player = PLAYER.lock().unwrap();
        match &mut *opt_player {
            None => *opt_player = Some(Player::new(username, PlayerSettings { num_rows_cols })),
            Some(player) => {
                (*player).username = username;
                (*player).player_settings.num_rows_cols = num_rows_cols;
            }
        }

        // Finish sittings iff player save worked
        if let Ok(_) = opt_player.as_ref().unwrap().save(ctx) {
            self.advance_scene = true;
        }

        Ok(())
    }
    pub fn new(ctx: &mut Context, intro: bool) -> GameResult<Self> {
        let title_fragment = TextFragment {
            text: "".to_string(),
            color: Some(Color::BLACK),
            font: Some("SecularOne-Regular".into()),
            scale: Some(PxScale::from(58.0)),
        };
        let welcome = Text::new(TextFragment {
            text: "Welcome to Sliding Puzzle.".to_string(),
            ..title_fragment.clone()
        });
        let w_sz = welcome.measure(ctx)?;
        Ok(SettingsScene {
            intro,
            intro_animation: keyframes![
                (0.0, 0.0, EaseInOut),
                (90.0, 1.0, EaseInOut),
                (90.0, 2.0, EaseInOut)
            ],
            options: vec![
                GameMenuItem::new_input_item(
                    ctx,
                    "Username",
                    false,
                    // Will never get called
                    Box::new(|_| panic!()),
                    0.0,
                    0.0, // Doesn't matter
                    w_sz.x,
                    INPUT_BOX_HEIGHT,
                )?,
                GameMenuItem::new_input_item(
                    ctx,
                    "Board Size",
                    true,
                    // Will never get called
                    Box::new(|_| panic!()),
                    0.0,
                    0.0, // Doesn't matter
                    w_sz.x,
                    INPUT_BOX_HEIGHT,
                )?,
            ],
            greeting_visible: false,
            greeting: Text::new(TextFragment {
                text: "Hi!".to_string(),
                ..title_fragment.clone()
            }),
            welcome,
            welcome_visible: false,
            main: Text::new(TextFragment {
                text: "Settings".to_string(),
                ..title_fragment.clone()
            }),
            selected_option: 0,
            menu_visible: false,
            enter_confirm: Text::new(TextFragment {
                text: "Press Enter to save.".to_string(),
                ..title_fragment.clone()
            }),
            enter_confirm_visible: false,
            advance_scene: false,
        })
    }
}

impl Drawable for SettingsScene {
    fn draw(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> ggez::GameResult {
        if self.intro {
            if !self.greeting_visible {
                self.intro_animation.advance_by(0.05);
                canvas.draw(&self.greeting, Vec2::new(self.intro_animation.now(), 90.0));
                if self.intro_animation.finished() {
                    self.greeting_visible = true;
                    self.intro_animation.advance_to(0.0);
                }
            } else {
                canvas.draw(&self.greeting, Vec2::new(90.0, 90.0));
            }

            if self.greeting_visible && !self.welcome_visible {
                self.intro_animation.advance_by(0.05);
                canvas.draw(
                    &self.welcome,
                    Vec2::new(
                        self.intro_animation.now(),
                        self.greeting.measure(ctx)?.y + 90.0,
                    ),
                );
                if self.intro_animation.finished() {
                    self.welcome_visible = true;
                    self.intro_animation.advance_to(0.0);
                }
            } else if self.greeting_visible && self.welcome_visible {
                canvas.draw(
                    &self.welcome,
                    Vec2::new(90.0, self.greeting.measure(ctx)?.y + 90.0),
                );

                for i in 0..self.options.len() {
                    // This should be done earlier
                    self.options[i].y = self.greeting.measure(ctx)?.y
                        + self.welcome.measure(ctx)?.y
                        + 120.0
                        + ((INPUT_BOX_HEIGHT + 20.0) * i as f32);
                    if !self.menu_visible {
                        self.intro_animation.advance_by(0.05);
                        self.options[i].x = self.intro_animation.now();
                    }
                    if self.intro_animation.finished() {
                        self.intro_animation.advance_to(0.0);
                        self.menu_visible = true;
                    }
                    self.options[i].draw(ctx, canvas)?;
                }
                if self.menu_visible {
                    canvas.draw(
                        &self.enter_confirm,
                        Vec2::new(
                            if self.enter_confirm_visible {
                                90.0
                            } else {
                                self.intro_animation.advance_by(0.05);
                                let p = self.intro_animation.now();
                                if self.intro_animation.finished() {
                                    self.intro_animation.advance_to(0.0);
                                    self.enter_confirm_visible = true;
                                    self.options[self.selected_option].select();
                                }

                                p
                            },
                            self.greeting.measure(ctx)?.y
                                + self.welcome.measure(ctx)?.y
                                + 170.0
                                + ((INPUT_BOX_HEIGHT + 20.0) * self.options.len() as f32),
                        ),
                    )
                }
            }
        }
        Ok(())
    }
}
impl Scene for SettingsScene {
    fn text_input_event(&mut self, _ctx: &mut ggez::Context, c: char) {
        // Will always be the case (for now). We don't want spaces.
        // This is not the best solution, but it is a solution.
        if c != ' ' {
            self.options[self.selected_option].handle_input(c)
        }
    }
    fn handle_key_event(
        &mut self,
        ctx: &mut ggez::Context,
        key_input: ggez::input::keyboard::KeyInput,
        _repeat: bool,
    ) {
        if let Some(vkeycode) = key_input.keycode {
            let old_option = self.selected_option;
            match vkeycode {
                KeyCode::Up => {
                    if self.selected_option > 0 {
                        self.selected_option -= 1;
                    }
                }
                KeyCode::Down => {
                    if self.selected_option < self.options.len() - 1 {
                        self.selected_option += 1;
                    }
                }
                KeyCode::Return => {
                    let mut valid_inputs = true;
                    for option in &mut self.options {
                        // Always unwrap since all of them are input boxes
                        if option.get_input_value().unwrap().is_empty() {
                            valid_inputs = false
                        }
                    }
                    if valid_inputs {
                        self.save_configuration(ctx)
                            .expect("Failed to save configuration");
                    }
                }
                _ => {}
            }

            if old_option != self.selected_option {
                self.options[old_option].deselect();
                self.options[self.selected_option].select();
            }
        }
    }

    fn next_scene(&mut self, ctx: &mut ggez::Context) -> Option<Box<dyn Scene>> {
        if self.advance_scene {
            Some(Box::new(
                GameMenu::new(ctx).expect("Failed to create game menu"),
            ))
        } else {
            None
        }
    }

    fn draw_transition(
        &mut self,
        ctx: &mut ggez::Context,
        canvas: &mut ggez::graphics::Canvas,
    ) -> ggez::GameResult {
        if !self.intro {
            self.draw(ctx, canvas)?;
        }
        Ok(())
    }
}
