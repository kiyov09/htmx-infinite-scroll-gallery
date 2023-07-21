use axum::{extract::Query, routing::get, Router};
use std::{collections::HashMap, fmt::Display, net::SocketAddr, str::FromStr};

use tower_http::services::ServeDir;

use leptos::*;
use leptos_meta::*;

//
// Server setup
//
#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/", get(root))
        .route("/more", get(more))
        .route("/modal/open", get(modal))
        .nest_service("/static", ServeDir::new("static"));

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

//
// Types
//

// Indicates the direction of the navigation when
// the modal is open
enum Direction {
    Left,
    Right,
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Direction::Left => write!(f, "left"),
            Direction::Right => write!(f, "right"),
        }
    }
}

impl FromStr for Direction {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "left" => Ok(Direction::Left),
            "right" => Ok(Direction::Right),
            _ => Err(()),
        }
    }
}

//
// API
//

//
// Serves the index page (app shell)
//
async fn root() -> axum::response::Html<String> {
    leptos::ssr::render_to_string(|cx| {
        view! {
            cx,
            <App />
        }
    })
    .into()
}

//
// Serves the requests for more images, which are triggered
// by the intersection observer feature
//
async fn more() -> axum::response::Html<String> {
    leptos::ssr::render_to_string(|cx| {
        view! {
            cx,
            <ImageList />
        }
    })
    .into()
}

// Serves the requests for the modal
// The query parameters are used to determine the direction
// of the navigation and the image to display
//
async fn modal(Query(q): Query<HashMap<String, String>>) -> axum::response::Html<String> {
    let url = q.get("url").unwrap_or(&String::from("")).to_string();
    let dir = q.get("dir").and_then(|dir| dir.parse::<Direction>().ok());

    leptos::ssr::render_to_string(|cx| {
        view! {
            cx,
            <Modal url dir />
        }
    })
    .into()
}

//
// Leptos components
//

//
// Provides the app shell + the initial set of images
//
#[component]
pub fn app(cx: Scope) -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    // This is not working, I need to investigate why
    provide_meta_context(cx);

    let title = "HTMX Infinite Scroll Gallery";

    view! {
        cx,

        <head>
            // <Title text={title}/>
            <title>{title}</title>

            // <Stylesheet href="/static/output.css"/>
            <link rel="stylesheet"  href="/static/output.css"/>

        </head>

        // content for this welcome page
        <body class="max-w-7xl m-auto px-8 lg:px-12 pb-12 pt-20 bg-gray-200 font-poppins">
            <main class="w-full flex flex-col items-center gap-2 lg:gap-4 space-y-10">
                <h1 class="text-5xl tracking-wide font-semibold">{title}</h1>
                <ul id="images" class="w-full grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-3" >
                    <ImageList />
                </ul>
            </main>

            // HTMX
            <script src="https://unpkg.com/htmx.org@1.9.3/dist/htmx.min.js"></script>
        </body>
    }
}

//
// ImageList provides 16 images + an invisible element
// which'll be used as a target for the intersection observer.
// Also, it provides the indicator component, which is used
// to show a loading indicator while the next set of images
// is being fetched.
//
#[component]
fn image_list(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <For
            each = move || (0..16)
            key = |i| *i
            view = move |cx, _| {
                view! {
                    cx,
                    <ImageItem />
                }
            }
        />
        <li
            class="w-auto h-auto overflow-hidden flex rounded-xl mt-4 col-span-full justify-center"
            id="indicator-container"
            hx-trigger="intersect delay:0.75s"
            hx-get="/more"
            hx-target="this"
            hx-swap="outerHTML"
        >
            <Indicator />
        </li>
    }
}

//
// Represents a single image
// Once cliked, it'll request the modal to be opened
//
#[component]
fn image_item(cx: Scope) -> impl IntoView {
    let url = random_image_url();

    view! {
        cx,
        <li
            tabindex=1
            class="w-auto h-auto overflow-hidden flex rounded-xl shadow-md bg-gray-100 group hover:ring-2 hover:ring-neutral-400 hover:ring-offset-2 focus:ring-2 focus:ring-neutral-400 focus:ring-offset-2 cursor-pointer outline-none"
            hx-trigger="click, keyup[key=='Enter']"
            hx-get=format!("/modal/open?url={url}")
            hx-target="body"
            hx-swap="beforeend"
        >
            <img
                class="w-full h-full object-cover aspect-square transition duration-[2s] group-hover:scale-110 group-focus:scale-110"
                src=url alt=""
            />
        </li>
    }
}

//
// Modal component to display the image in a larger size
// It also provides buttons to navigate to the next/previous image
//
#[component]
fn modal(cx: Scope, url: String, dir: Option<Direction>) -> impl IntoView {
    let (base, id) = url.split_once('?').unwrap();
    let id = id.parse::<i32>().unwrap();

    let modal_id = match dir {
        Some(dir) => format!("modal-content-{}", dir),
        None => "modal-content".to_string(),
    };

    view! {
        cx,
        <div class="fixed w-full h-full top-0 left-0 focus:opacity-75 overflow-hidden" hx-target="this" hx-swap="outerHTML">
            // Backgrop
            <div class="w-full h-full bg-gray-800 opacity-75"
                hx-on="click: this.parentElement.outerHTML = ''"
            ></div>

            // Nav buttons
            // Left
            <button
                class="fixed text-2xl top-1/2 -translate-y-1/2 left-10 cursor-pointer text-white p-2 aspect-square rounded-full ring-1 ring-gray-50 active:bg-gray-500"
                hx-trigget="click"
                hx-get=format!("/modal/open?dir=left&url={base}?{}", id - 1)
            >
                <svg fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M15.75 19.5L8.25 12l7.5-7.5" />
                </svg>
            </button>

            // Right
            <button
                class="fixed text-2xl top-1/2 -translate-y-1/2 right-10 cursor-pointer text-white p-2 aspect-square rounded-full ring-1 ring-gray-50 active:bg-gray-500"
                hx-trigget="click"
                hx-get=format!("/modal/open?dir=right&url={base}?{}", id + 1)
            >
                <svg fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                </svg>
            </button>

            // Actual modal
            <div
                // id="modal-content"
                id=modal_id
                class="fixed top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 w-4/5 lg:w-1/2 max-w-3xl aspect-square rounded-md shadow-md overflow-hidden outline-none"
                tabindex="1"
                autofocus
            >
                <img
                    class="w-full h-full object-cover aspect-square"
                    src=url alt=""
                />
            </div>

            // Close
            <button
                class="fixed top-6 right-6 rounded-full bg-white shadow-xl w-8 h-8 flex items-center justify-center font-light text-xl text-neutral-700 cursor-pointer"
                hx-on="click: this.parentElement.outerHTML = ''"
            >
                <svg fill="none" viewBox="0 0 24 24" stroke-width="1.5" stroke="currentColor" class="w-6 h-6">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                </svg>
            </button>
        </div>
    }
}

//
// Loading indicator for when the next set of images is being fetched
//
#[component]
fn indicator(cx: Scope) -> impl IntoView {
    view! {
        cx,
        <div class="indicator text-3xl">
            <span class="inline-block animate-bounce">"."</span>
            <span class="inline-block animate-bounce">"."</span>
            <span class="inline-block animate-bounce">"."</span>
        </div>
    }
}

//
// Utils
//

static mut IMAGE: u32 = 0;

fn random_image_url() -> String {
    unsafe { IMAGE += 1 };
    format!("https://picsum.photos/800/800?{}", unsafe { IMAGE })
}
