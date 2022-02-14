#![feature(array_zip)]

mod keyboard;
mod wordle;
mod game;

use stylist::{global_style, GlobalStyle};
use yew::prelude::*;

use keyboard::Keyboard;
use crate::keyboard::Key;
use game::Game;

struct App;

impl Component for App {
    type Message = ();
    type Properties = ();

    fn create(ctx: &Context<Self>) -> Self {
        Self
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        html! {
            <>
                <Game />
            </>
        }
    }
}

fn style() -> GlobalStyle {
    global_style!(r#"
        body {
            font-family: 'Clear Sans', 'Helvetica Neue', Arial, sans-serif;
            text-align: center;
            max-width: 500px;
            margin: 0px auto;
        }
    "#).expect("")
}

fn main() {
    style();
    yew::start_app::<App>();
}
