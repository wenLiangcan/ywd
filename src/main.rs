#![feature(array_zip)]
#![feature(if_let_guard)]

mod keyboard;
mod wordle;
mod game;

use gloo::events::EventListener;
use gloo::utils::{document, window};
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

fn on_window_resize() {
    if let Some(body) = document().body() {
        if let Ok(inner_height) = window().inner_height() {
            body.style()
                .set_property("--vh",
                              format!("{}px", inner_height.as_f64().unwrap()).as_str()).unwrap();
        }
    }
}

fn main() {
    style();
    on_window_resize();
    EventListener::new(&window(), "resize", |_| {on_window_resize()})
        .forget();
    yew::start_app::<App>();
}
