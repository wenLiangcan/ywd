use wasm_bindgen::JsCast;
use std::fmt;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;
use gloo::events::EventListener;
use gloo::timers::callback::Timeout;
use gloo::utils::window;
use stylist::{css, StyleSource, YieldStyle};
use yew::{classes, Component, Context, Html};
use yew::html::Scope;
use yew::prelude::*;
use web_sys::KeyboardEvent;
use crate::game::GameState::InProgress;
use crate::game::LetterState::Initial;
use crate::Key;
use crate::wordle::{LetterHint, Wordle};
use crate::Keyboard;

#[derive(Copy, Clone, PartialEq, Debug)]
enum LetterState {
    Initial,
    Hint(LetterHint),
}

impl fmt::Display for LetterState {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Hint(hint) => format!("{:?}", hint),
            s => format!("{:?}", s),
        })
    }
}

pub enum Msg {
    Press(Key),
    ShowMessage(&'static str, u32),
    ClearMessage,
    Shake,
    StopShaking,
}

enum GameState {
    InProgress,
    Failed,
    Wined,
}

pub struct Game {
    state: GameState,
    wordle: Wordle,
    guesses: Vec<[(char, LetterState); 5]>,
    current_guess: Vec<char>,
    letter_states: Rc<RefCell<HashMap<char, Option<LetterHint>>>>,
    message: String,
    shake: bool,
}

impl Component for Game {
    type Message = Msg;
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        let state_map = ('a' ..= 'z')
            .fold(HashMap::new(), |mut m, c| {
                m.insert(c, None);
                m
            });
        Self {
            state: InProgress,
            wordle: Wordle::new_of_the_day(),
            guesses: vec![],
            current_guess: vec![],
            letter_states: Rc::new(RefCell::new(state_map)),
            message: "".to_string(),
            shake: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        let link = ctx.link();
        match self.state {
            GameState::InProgress => (),
            _ => return false,
        }
        match msg {
            Self::Message::Press(key) => match key {
                _ if self.guesses.len() == 6 => {
                    false
                },
                Key::Backspace if self.current_guess.len() > 0 => {
                    self.current_guess.pop();
                    true
                },
                Key::Letter(c) if self.current_guess.len() < 5 => {
                    self.current_guess.push(c);
                    true
                },
                Key::Enter if self.current_guess.len() < 5 => {
                    self.shake(link);
                    self.show_message(link, "Not enough letters", 1000);
                    false
                },
                Key::Enter if self.current_guess.len() == 5 => {
                    let guess: [char; 5] = self.current_guess.as_slice().try_into().unwrap();
                    let result = self.wordle.guess(guess);
                    match result {
                        Ok(hints) => {
                            self.current_guess.clear();

                            hints.iter().for_each(|(c, s)| {
                                self.letter_states.borrow_mut().entry(*c).and_modify(|state| {
                                    match state {
                                        None => {
                                            state.replace(s.clone());
                                        },
                                        _ => (),
                                    }
                                });
                            });

                            self.guesses.push(hints
                                .map(|(c, h)| {(c, LetterState::Hint(h))}));

                            if hints.iter().all(|(_, h)| {*h == LetterHint::Correct}) {
                                self.state = GameState::Wined;
                                self.message = "win !!!".to_string();  // todo
                            } else if self.guesses.len() == 6 {
                                self.state = GameState::Failed;
                                self.message = self.wordle.get_answer().to_string();
                            }

                            true
                        },
                        _ => {
                            self.shake(link);
                            self.show_message(link, "Not in word list", 1000);
                            false
                        },
                    }
                },
                _ => false,
            },
            Self::Message::ClearMessage => {
                self.message = "".to_string();
                true
            },
            Self::Message::ShowMessage(msg, timeout) => {
                self.message = msg.to_string();
                let link = ctx.link().clone();
                Timeout::new(timeout, move || {link.send_message(Self::Message::ClearMessage)})
                    .forget();
                true
            },
            Self::Message::StopShaking => {
                self.shake = false;
                true
            },
            Self::Message::Shake => {
                self.shake = true;
                let link = ctx.link().clone();
                Timeout::new(1000, move || {link.send_message(Self::Message::StopShaking)})
                    .forget();
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let on_key_pressed = ctx.link().callback(|key| {Self::Message::Press(key)});
        let letter_states = Rc::clone(&self.letter_states);
        let rows = self.build_rows();
        html! {
            <div class={self.style()}>
                {self.view_message()}
                <header>
                    <h1>{"YDW"}</h1>
                </header>
                {self.view_board(rows)}
                <Keyboard on_key_pressed={on_key_pressed} letter_states={letter_states}/>
            </div>
        }
    }

    fn rendered(&mut self, ctx: &Context<Self>, first_render: bool) {
        if first_render {
            let onkeyup = ctx.link().batch_callback(|e: KeyboardEvent| {
                match e.key().parse::<Key>() {
                    Ok(key) => Some(Self::Message::Press(key)),
                    _ => None
                }
            });
            EventListener::new(&window(), "keyup", move |e| {
                let e = e.clone();
                onkeyup.emit(e.dyn_into::<KeyboardEvent>().expect(""))
            }).forget();
        }
    }
}

impl Game {
    fn build_rows(&self) -> [[(char, LetterState); 5]; 6] {
        let mut rows = [[(' ', Initial); 5]; 6];
        for (i, g) in self.guesses.iter().enumerate() {
            rows[i] = *g;
        }
        if self.guesses.len() < 6 {
            let mut current_guess = [(' ', Initial); 5];
            for (i, c) in self.current_guess.iter().enumerate() {
                current_guess[i].0 = *c;
            }
            rows[self.guesses.len()] = current_guess;
        }
        rows
    }

    fn show_message(&self, link: &Scope<Self>, message: &'static str, millis: u32) {
        link.send_message(<Self as Component>::Message::ShowMessage(message, millis));
    }

    fn shake(&self, link: &Scope<Self>) {
        link.send_message(<Self as Component>::Message::Shake);
    }

    fn view_message(&self) -> Html {
        match self.message.as_str() {
            "" => html! {},
            msg => html! {
                <div class="message">{msg}</div>
            },
        }
    }

    fn view_board(&self, rows: [[(char, LetterState); 5]; 6]) -> Html {
        html! {
            <div id="board">{
                rows.iter().enumerate().map(|(row_num, c2s)| {
                    let shake_row_class = if self.shake && row_num == self.guesses.len() {
                        Some("shake")
                    } else { None };
                    html! {
                        <div class={classes!("row", shake_row_class)}>{
                            c2s.iter().map(|(c, s)| {
                                let state_class = format!("{}", s);
                                let filled_class = if *c == ' '{ None } else { Some("filled") };
                                let revealed_class = if let LetterState::Initial = s {
                                    None
                                } else { Some("revealed") };
                                html! {
                                    <div class={classes!("tile", filled_class, revealed_class)}>
                                        <div class="front">{c}</div>
                                        <div class={classes!("back", state_class)}>{c}</div>
                                    </div>
                                }
                            }).collect::<Html>()
                        }</div>
                    }
                }).collect::<Html>()
            }</div>
        }
    }
}

impl YieldStyle for Game {
    fn style_from(&self) -> StyleSource<'static> {
        css!(r#"
            #board {
                display: grid;
                grid-template-rows: repeat(6, 1fr);
                grid-gap: 5px;
                padding: 10px;
                box-sizing: border-box;
                --height: min(420px, calc(var(--vh, 100vh) - 310px));
                height: var(--height);
                width: min(350px, calc(var(--height) / 6 * 5));
                margin: 0px auto;
            }
            .message {
                position: absolute;
                left: 50%;
                top: 80px;
                color: #fff;
                background-color: rgba(0, 0, 0, 0.85);
                padding: 16px 20px;
                z-index: 2;
                border-radius: 4px;
                transform: translateX(-50%);
                transition: opacity 0.3s ease-out;
                font-weight: 600;
            }
            .row {
                display: grid;
                grid-template-columns: repeat(5, 1fr);
                grid-gap: 5px;
            }
            .tile {
                width: 100%;
                font-size: 2rem;
                line-height: 2rem;
                font-weight: bold;
                vertical-align: middle;
                text-transform: uppercase;
                user-select: none;
                position: relative;
            }
            .tile.filled {
                animation: zoom 0.2s;
            }
            .tile .front,
            .tile .back {
                box-sizing: border-box;
                display: inline-flex;
                justify-content: center;
                align-items: center;
                position: absolute;
                top: 0;
                left: 0;
                width: 100%;
                height: 100%;
                transition: transform 0.6s;
                backface-visibility: hidden;
                -webkit-backface-visibility: hidden;
            }
            .tile .front {
                border: 2px solid #d3d6da;
            }
            .tile.filled .front {
                border-color: #999;
            }
            .tile .back {
                transform: rotateX(180deg);
            }
            .tile.revealed .front {
                transform: rotateX(180deg);
            }
            .tile.revealed .back {
                transform: rotateX(0deg);
            }

            @keyframes zoom {
                0% {
                    transform: scale(1.1);
                }
                100% {
                    transform: scale(1);
                }
            }
            .shake {
                animation: shake 0.5s;
            }
            @keyframes shake {
                0% {
                    transform: translate(1px);
                }
                10% {
                    transform: translate(-2px);
                }
                20% {
                    transform: translate(2px);
                }
                30% {
                    transform: translate(-2px);
                }
                40% {
                    transform: translate(2px);
                }
                50% {
                    transform: translate(-2px);
                }
                60% {
                    transform: translate(2px);
                }
                70% {
                    transform: translate(-2px);
                }
                80% {
                    transform: translate(2px);
                }
                90% {
                    transform: translate(-2px);
                }
                100% {
                    transform: translate(1px);
                }
            }

            @media (max-height: 680px) {
                .tile {
                    font-size: 3vh;
                }
            }

            h1 {
                margin: 4px 0;
                font-size: 36px;
            }

            header {
                border-bottom: 1px solid #ccc;
                margin-bottom: 30px;
                position: relative;
            }

            .Correct,
            .Present,
            .Absent {
                color: #fff !important;
            }

            .Correct {
                background-color: #6aaa64 !important;
            }

            .Present {
                background-color: #c9b458 !important;
            }

            .Absent {
                background-color: #787c7e !important;
            }
        "#)
    }
}