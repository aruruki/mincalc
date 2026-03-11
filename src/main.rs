use dioxus::prelude::*;

fn main() {
    launch(App);
}

#[component]
fn App() -> Element {
    rsx! {
        document::Stylesheet { href: asset!("/assets/tailwind.css") }
        div { class: "min-h-screen bg-gray-950 text-gray-100 flex flex-col",
            Header {}
            main { class: "flex-1 container mx-auto px-4 py-12 flex items-center justify-center",
                div { class: "text-center",
                    h2 { class: "text-2xl font-semibold text-gray-300 mb-4",
                        "Water Concentrate Builder"
                    }
                    p { class: "text-gray-500 max-w-md",
                        "Calculate mineral additions to craft the perfect water profile for your coffee."
                    }
                }
            }
            Footer {}
        }
    }
}

#[component]
fn Header() -> Element {
    rsx! {
        header { class: "border-b border-gray-800 bg-gray-900",
            div { class: "container mx-auto px-4 py-4 flex items-center gap-3",
                div { class: "w-8 h-8 rounded-full bg-blue-500 flex items-center justify-center text-sm font-bold",
                    "M"
                }
                h1 { class: "text-lg font-semibold tracking-tight", "mincalc" }
                span { class: "text-gray-500 text-sm", "/ water concentrate calculator" }
            }
        }
    }
}

#[component]
fn Footer() -> Element {
    rsx! {
        footer { class: "border-t border-gray-800 bg-gray-900",
            div { class: "container mx-auto px-4 py-3 text-center text-gray-600 text-xs",
                "mincalc — fully client-side, no data leaves your browser"
            }
        }
    }
}
