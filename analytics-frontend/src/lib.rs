use wasm_bindgen::prelude::wasm_bindgen;
use yew::prelude::*;
use yew_router::prelude::*;

mod components;
mod pages;
mod services;
mod types;

use components::layout::Layout;
use pages::{analytics::Analytics, home::Home, trades::Trades};

#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/analytics")]
    Analytics,
    #[at("/trades")]
    Trades,
}

fn switch(routes: Route) -> Html {
    match &routes {
        Route::Home => html! { <Home /> },
        Route::Analytics => html! { <Analytics /> },
        Route::Trades => html! { <Trades /> },
    }
}

#[function_component(App)]
fn app() -> Html {
    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}

#[wasm_bindgen(start)]
pub fn main() {
    wasm_logger::init(wasm_logger::Config::default());

    yew::Renderer::<App>::new().render();
}
