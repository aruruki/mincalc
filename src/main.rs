mod chemistry;
mod components;
mod state;
mod storage;

use dioxus::prelude::*;
use state::{AppState, Tab};

use components::{
    concentrate_builder::ConcentrateBuilder,
    recipe_builder::RecipeBuilder,
};

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    // Single global signal holding all app state.
    // Hydrate concentrates from localStorage if available.
    use_context_provider(|| {
        let mut state = AppState::default();
        if let Some((concentrates, next_id)) = storage::load_concentrates() {
            state.concentrates = concentrates;
            state.next_id = next_id;
        }
        Signal::new(state)
    });

    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div { class: "min-h-screen bg-gray-950 text-gray-100 flex flex-col",
            AppHeader {}
            TabBar {}
            main { class: "flex-1 container mx-auto px-4 py-6 max-w-4xl",
                TabContent {}
            }
            AppFooter {}
        }
    }
}

// ---------------------------------------------------------------------------
// Header
// ---------------------------------------------------------------------------

#[component]
fn AppHeader() -> Element {
    rsx! {
        header { class: "border-b border-gray-800 bg-gray-900",
            div { class: "container mx-auto px-4 py-4 max-w-4xl flex items-center gap-3",
                div { class: "w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-sm font-bold shrink-0",
                    "M"
                }
                div {
                    h1 { class: "text-lg font-semibold tracking-tight leading-tight", "mincalc" }
                    p { class: "text-gray-500 text-xs leading-tight", "water concentrate calculator" }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tab bar
// ---------------------------------------------------------------------------

#[component]
fn TabBar() -> Element {
    let mut state = use_context::<Signal<AppState>>();
    let active = state.read().active_tab;

    let tab_class = |t: Tab| {
        let base = "px-5 py-3 text-sm font-medium border-b-2 transition-colors";
        if active == t {
            format!("{} border-blue-500 text-blue-400", base)
        } else {
            format!("{} border-transparent text-gray-500 hover:text-gray-300", base)
        }
    };

    rsx! {
        nav { class: "border-b border-gray-800 bg-gray-900",
            div { class: "container mx-auto px-4 max-w-4xl flex",
                button {
                    class: "{tab_class(Tab::Concentrates)}",
                    onclick: move |_| state.write().active_tab = Tab::Concentrates,
                    "Concentrates"
                }
                button {
                    class: "{tab_class(Tab::Recipe)}",
                    onclick: move |_| state.write().active_tab = Tab::Recipe,
                    "Recipe"
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Tab content
// ---------------------------------------------------------------------------

#[component]
fn TabContent() -> Element {
    let state = use_context::<Signal<AppState>>();
    let active = state.read().active_tab;

    match active {
        Tab::Concentrates => rsx! { ConcentrateBuilder {} },
        Tab::Recipe => rsx! { RecipeBuilder {} },
    }
}

// ---------------------------------------------------------------------------
// Footer
// ---------------------------------------------------------------------------

#[component]
fn AppFooter() -> Element {
    rsx! {
        footer { class: "border-t border-gray-800 bg-gray-900",
            div { class: "container mx-auto px-4 py-3 max-w-4xl text-center text-gray-600 text-xs",
                "mincalc — fully client-side, all calculations run in your browser"
            }
        }
    }
}
