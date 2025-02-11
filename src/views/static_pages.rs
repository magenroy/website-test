use crate::prelude::*;
use leptos_router::{
    components::Route,
    path,
    static_routes::StaticRoute,
    MatchNestedRoutes, SsrMode,
};
use std::path::PathBuf;

#[component(transparent)]
pub fn Routes() -> impl MatchNestedRoutes + Clone {
    // use std::path::Path;
    view! {
            <Route
                path=path!("/")
                view=HomePage
                ssr=SsrMode::Static(
                    StaticRoute::new()
                    // .regenerate(|_| watch_path(Path::new("./posts"))),
                )
            />

            <Route
                path=path!("/:slug")
                view=ReadFile
                ssr=SsrMode::Static(
                    StaticRoute::new()
                        .prerender_params(|| async move {
                            [("slug".into(), list_slugs("./static/".into(), "html".into()).await.unwrap_or_default())]
                                .into_iter()
                                .collect()
                        })
                        /* .regenerate(|params| {
                            let slug = params.get("slug").unwrap();
                            watch_path(Path::new(&format!("./posts/{slug}.md")))
                        }), */
                )
            />

    }
    .into_inner()
}

#[component]
fn ReadFile() -> impl IntoView {
    let slug = leptos_router::hooks::use_params_map().read().get("slug").unwrap_or_default();
    let mut filename = PathBuf::from(&slug);

    println!("{}", &slug);

    if let Some(ext) = filename.extension() {
        println!("{:?}", ext);
        if ext == std::ffi::OsStr::new("html") {
            use leptos_router::components::Redirect;
            filename.set_extension("");
            let new_slug = String::from(filename.to_string_lossy());
            return view! { <Redirect path=new_slug/> }.into_any()
            // FIX: works the first time, but the second time it doesn't redirect?? And it's an
            // empty page for some reason??
        } else {
            return view! { <p> "What is this file?" </p> }.into_any()
        }
        println!("{:?}", &filename);
    }

    filename.set_extension("html");

    let mut path = PathBuf::from("./static/");
    path.push(&filename);
    // let path = format!("./static/{slug}.html");

    // Since I intend to just use the server to produce static pages, I can just read the file
    // (synchronously!) on the server in order to produce the page
    let content = std::fs::read_to_string(path).unwrap_or(format!("Unable to read file: {:?}", &filename));

    view! {
        <div
            inner_html=content
            style:font-family="times, serif"
        >
        </div>
    }.into_any()
}

#[component]
fn HomePage() -> impl IntoView {
    let slugs = Resource::new(|| (), |_| list_slugs("./static/".into(), "html".into()));
    let slugs = move || {
        slugs
            .get()
            .map(|n| n.unwrap_or_default())
            .unwrap_or_default()
    };

    use leptos_router::components::A;
    view! {
        <h1>"Static pages"</h1>
        <Suspense fallback=move || view! { <p>"Loading pages..."</p> }>
            <ul>
                <For each=slugs key=|slug| slug.clone() let:slug>
                    <li>
                        <A href={slug.clone()}>{slug.clone()}</A>
                    </li>
                </For>
            </ul>
        </Suspense>
    }
}
