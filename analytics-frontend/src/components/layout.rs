use crate::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct LayoutProps {
    pub children: Children,
}

#[function_component(Layout)]
pub fn layout(props: &LayoutProps) -> Html {
    html! {
        <div class="min-h-screen bg-gray-100">
            <nav class="bg-blue-600 shadow-lg">
                <div class="max-w-7xl mx-auto px-4">
                    <div class="flex justify-between h-16">
                        <div class="flex items-center">
                            <h1 class="text-white text-xl font-bold">
                                {"Crypto Analytics Dashboard"}
                            </h1>
                        </div>
                        <div class="flex items-center space-x-4">
                            <Link<Route> to={Route::Home} classes="text-white hover:text-blue-200 px-3 py-2 rounded-md text-sm font-medium">
                                {"Home"}
                            </Link<Route>>
                            <Link<Route> to={Route::Analytics} classes="text-white hover:text-blue-200 px-3 py-2 rounded-md text-sm font-medium">
                                {"Analytics"}
                            </Link<Route>>
                            <Link<Route> to={Route::Trades} classes="text-white hover:text-blue-200 px-3 py-2 rounded-md text-sm font-medium">
                                {"Live Trades"}
                            </Link<Route>>
                        </div>
                    </div>
                </div>
            </nav>
            <main class="max-w-7xl mx-auto py-6 px-4 sm:px-6 lg:px-8">
                {props.children.clone()}
            </main>
        </div>
    }
}
