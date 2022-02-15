use std::{fmt, iter};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::rc::Rc;
use std::str::FromStr;
use stylist::{css, StyleSource, YieldStyle};
use yew::prelude::*;
use yew::{classes, Html, Properties};
use crate::Key::Letter;
use crate::wordle::LetterHint;

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Key {
    Letter(char),
    Enter,
    Backspace,
}

impl FromStr for Key {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            _ if s.len() == 1 => match s.chars().nth(0) {
                Some(c@'a' ..= 'z') => Ok(Key::Letter(c)),
                _ => Err(()),
            },
            "Enter" => Ok(Key::Enter),
            "Backspace" => Ok(Key::Backspace),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", match self {
            Self::Letter(l) => l.to_string(),
            k => format!("{:?}", k),
        })
    }
}

pub struct Keyboard;

#[derive(Properties, PartialEq)]
pub struct KeyboardProperties {
    pub on_key_pressed: Callback<Key>,
    pub letter_states: Rc<RefCell<HashMap<char, Option<LetterHint>>>>,  // todo make it immutable
}

impl Component for Keyboard {
    type Message = ();
    type Properties = KeyboardProperties;

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let rows: Vec<Vec<Key>> = vec![
            "qwertyuiop".chars().map(Key::Letter).collect(),
            "asdfghjkl".chars().map(Key::Letter).collect(),
            iter::once(Key::Enter)
                .chain("zxcvbnm".chars().map(Key::Letter))
                .chain(iter::once(Key::Backspace)).collect(),
        ];
        html! {
            <div class={self.style()}>{
                rows.iter().enumerate().map(|(i, row)| {
                    html! {
                        <div class="row">{
                            iter::once(self.view_spacer(i == 1)).chain(row.iter().map(|&key| {
                                let on_key_pressed = ctx.props().on_key_pressed.clone();
                                let state = match &key {
                                    Key::Letter(c) =>
                                        (*ctx.props().letter_states).borrow().get(c)
                                            .and_then(|s| {s.as_ref()}).map(|h| {h.clone()}),
                                    _ => None,
                                };
                                self.view_button(key.clone(), state, on_key_pressed)
                            })).chain(iter::once(self.view_spacer(i == 1))).collect::<Html>()
                        }</div>
                    }
                }).collect::<Html>()
            }</div>
        }
    }
}

impl Keyboard {
    fn view_backspace(&self) -> Html {
        html! {
            <svg xmlns="http://www.w3.org/2000/svg" height="24" viewBox="0 0 24 24" width="24">
                <path
                    fill="currentColor"
                    d="M22 3H7c-.69 0-1.23.35-1.59.88L0 12l5.41 8.11c.36.53.9.89 1.59.89h15c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm0 16H7.07L2.4 12l4.66-7H22v14zm-11.59-2L14 13.41 17.59 17 19 15.59 15.41 12 19 8.41 17.59 7 14 10.59 10.41 7 9 8.41 12.59 12 9 15.59z">
                </path>
            </svg>
        }
    }

    fn view_button(&self, key: Key, state: Option<LetterHint>, press_callback: Callback<Key>) -> Html {
        let big_key_class = if let Letter(_) = key { None } else { Some("big") };
        let state_class = state.map(|s| {format!("{:?}", s)});
        html! {
            <button class={classes!(big_key_class, state_class)}
                onclick={Callback::from(move |_| press_callback.emit(key))}>{
                match key {
                    Key::Backspace => self.view_backspace(),
                    key => html! {
                        <span>{format!("{}", key)}</span>
                    }
                }
            }</button>
        }
    }

    fn view_spacer(&self, needed: bool) -> Html {
        if needed {
            html! {<div class="spacer" />}
        } else {
            html! {}
        }
    }
}

impl YieldStyle for Keyboard {
    fn style_from(&self) -> StyleSource<'static> {
        css!(r#"
           margin: 30px 8px 0;
           user-select: none;
           .row {
               display: flex;
               width: 100%;
               margin: 0 auto 8px;
               touch-action: manipulation;
           }
           .spacer {
               flex: 0.5;
           }
           button {
               font-family: inherit;
               font-weight: bold;
               border: 0;
               padding: 0;
               margin: 0 6px 0 0;
               height: 58px;
               border-radius: 4px;
               cursor: pointer;
               user-select: none;
               background-color: #d3d6da;
               color: #1a1a1b;
               flex: 1;
               display: flex;
               justify-content: center;
               align-items: center;
               text-transform: uppercase;
               -webkit-tap-highlight-color: rgba(0, 0, 0, 0.3);
               transition: all 0.2s 1.5s;
           }
           button:last-of-type {
               margin: 0;
           }
           button.big {
               flex: 1.5;
           }
        "#)
    }
}
