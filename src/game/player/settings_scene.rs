use std::{cell::RefCell, rc::Rc};

use ggez::{input::keyboard::KeyCode, Context, GameResult};
use keyframe::{functions::EaseInOut, keyframes, AnimationSequence};

use crate::game::{
    animation::{
        animation::{Animation, AnimationData},
        DrawablePos,
    },
    drawable::Drawable,
    gmenu::{
        game_menu::GameMenu,
        main_menu::MainMenu,
        menu_item_list::{GameMenuItemList, NewGameMenuItemData, NewGameMenuItemDataVariant},
    },
    resources::theme::Theme,
    scene::Scene,
    ui::uitext::UIText,
};

use super::{Player, PlayerSettings, PLAYER};

pub struct SettingsScene {
    intro: bool,
    intro_animation: Option<Animation<DrawablePos>>,
    greeting: Rc<RefCell<UIText>>,
    enter_confirm: Rc<RefCell<UIText>>,
    welcome: Rc<RefCell<UIText>>,
    // Menu options
    options: Rc<RefCell<GameMenuItemList>>,

    advance_scene: bool,

    main: UIText,
}

const INPUT_BOX_HEIGHT: f32 = 110.0;

impl SettingsScene {
    pub fn save_configuration(&mut self, ctx: &mut Context) -> GameResult {
        // Should be safe to unwrap here due to prior parsing
        let mut options = self.options.borrow_mut();
        let username = options.items[0].get_input_value().unwrap();
        let num_rows_cols = options.items[1].get_input_value().unwrap().parse().unwrap();

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
        let greeting = Rc::new(RefCell::new(UIText::new("Hi!".to_string(), Theme::fg_color(), 58.8, DrawablePos { x: 90.0, y: 90.0 })));
        let g_sz = greeting.borrow().text.measure(ctx)?;

        let welcome = Rc::new(RefCell::new(UIText::new(
            "Welcome to Sliding Puzzle!".to_string(),
            Theme::fg_color(),
            58.8,
            DrawablePos { x: 90.0, y: g_sz.y + 90.0 },
        )));
        let w_sz = welcome.borrow().text.measure(ctx)?;

        let opt_player = PLAYER.lock().unwrap();

        let o_y = g_sz.y + if intro { w_sz.y } else { 0.0 } + 120.0;
        let options = Rc::new(RefCell::new(GameMenuItemList::new(
            ctx,
            vec![
                NewGameMenuItemData {
                    variant: NewGameMenuItemDataVariant::InputItem {
                        prompt: "Username".to_string(),
                        is_num: false,
                        initial_value: if let Some(player) = opt_player.as_ref() { player.username.clone() } else { "".to_string() },
                    },
                    next_page: None,
                },
                NewGameMenuItemData {
                    variant: NewGameMenuItemDataVariant::InputItem {
                        prompt: "Board Size".to_string(),
                        is_num: true,
                        initial_value: if let Some(player) = opt_player.as_ref() {
                            format!("{}", player.player_settings.num_rows_cols)
                        } else {
                            "".to_string()
                        },
                    },
                    next_page: None,
                },
            ],
            90.0,
            o_y,
            w_sz.x,
            INPUT_BOX_HEIGHT,
        )?));

        let enter_confirm = Rc::new(RefCell::new(UIText::new(
            "Press Enter to Confirm.".to_string(),
            Theme::fg_color(),
            58.8,
            DrawablePos { x: 90.0, y: o_y + 50.0 + options.borrow().height() },
        )));

        // This doesn't work.
        if intro {
            // options.items[0].deselect();
        }

        let anim_frames = |y: f32| {
            keyframes![
                // I realize this first one is kind of a hack.
                (DrawablePos { x: -2000.0, y }, 0.0, EaseInOut),
                (DrawablePos { x: 90.0, y }, 1.0, EaseInOut),
                (DrawablePos { x: 90.0, y }, 2.0, EaseInOut)
            ]
        };

        let greeting_y = greeting.clone().borrow().pos.y;
        let welcome_y = welcome.clone().borrow().pos.y;
        let enter_confirm_y = enter_confirm.clone().borrow().pos.y;
        Ok(SettingsScene {
            intro,
            intro_animation: if intro {
                Some(Animation::new(vec![
                    AnimationData::Sequence((greeting.clone(), anim_frames(greeting_y))),
                    AnimationData::Sequence((welcome.clone(), anim_frames(welcome_y))),
                    AnimationData::Sequence((options.clone(), anim_frames(o_y))),
                    AnimationData::Sequence((enter_confirm.clone(), anim_frames(enter_confirm_y))),
                ]))
            } else {
                None
            },
            greeting,
            enter_confirm,
            welcome,
            options,
            advance_scene: false,
            main: UIText::new("Settings".to_string(), Theme::fg_color(), 58.8, DrawablePos { x: 90.0, y: 90.0 }),
        })
    }
}

impl Drawable for SettingsScene {
    fn draw(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        if let Some(anim) = &mut self.intro_animation {
            if !anim.finished() {
                anim.advance(0.05);
            }
        }

        if self.intro {
            self.greeting.borrow_mut().draw(ctx, canvas)?;
            self.welcome.borrow_mut().draw(ctx, canvas)?;
        } else {
            self.main.draw(ctx, canvas)?;
        }
        self.options.borrow_mut().draw(ctx, canvas)?;
        self.enter_confirm.borrow_mut().draw(ctx, canvas)?;

        Ok(())
    }
}
impl Scene for SettingsScene {
    fn text_input_event(&mut self, ctx: &mut ggez::Context, c: char) {
        self.options.borrow_mut().text_input_event(ctx, c);
    }
    fn handle_key_event(&mut self, ctx: &mut ggez::Context, key_input: ggez::input::keyboard::KeyInput, repeat: bool) {
        if let Some(KeyCode::Return) = key_input.keycode {
            let mut valid_inputs = true;
            for option in &mut self.options.borrow_mut().items {
                // Always unwrap since all of them are input boxes
                if option.get_input_value().unwrap().is_empty() {
                    valid_inputs = false
                }
            }
            if valid_inputs {
                self.save_configuration(ctx).expect("Failed to save configuration");
            }
        }
        // TODO make sure to handle this only if the opening animations have finished
        self.options.borrow_mut().handle_key_event(ctx, key_input, repeat);
    }

    fn next_scene(&mut self, ctx: &mut ggez::Context) -> Option<Box<dyn Scene>> {
        if self.advance_scene {
            Some(Box::new(GameMenu::new::<MainMenu>(ctx).expect("Failed to create game menu")))
        } else {
            None
        }
    }

    fn draw_transition(&mut self, ctx: &mut ggez::Context, canvas: &mut ggez::graphics::Canvas) -> ggez::GameResult {
        if !self.intro || self.advance_scene {
            self.draw(ctx, canvas)?;
        }
        Ok(())
    }
}
