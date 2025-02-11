use crate::views;
use crate::prelude::*;

use leptos_meta::MetaTags;
use leptos_meta::*;
use leptos_router::{
    components::{Outlet, Redirect, ParentRoute, Route, Router, Routes},
    path,
    static_routes::StaticRoute,
    SsrMode,
};
//
// NOTE: remember to change this if changing domain name!
// const URL: &str = "https://magenroy.github.io/website-test/";

const DESCRIPTION: &str = "Website of Roy Magen";
const AUTHOR: &str = "Roy Magen";
const NAME: &str = "Roy Magen";

// REF: https://github.com/leptos-rs/leptos/discussions/3039
// See https://github.com/leptos-rs/leptos/pull/1649?

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8"/>
                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                <Meta name="description" content=DESCRIPTION/>
                <Meta name="author" content=AUTHOR/>
                <Meta itemprop="description" content=DESCRIPTION/>
                <Meta itemprop="name" content=NAME/>
                // <Link rel="canonical" href=URL/>
                <AutoReload options=options.clone()/>
                <HydrationScripts options islands=true/>
                <MetaTags/>
            </head>
            <body>
                <App/>
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let fallback = || view! { "Page not found." }.into_view();

    view! {
        <Stylesheet href=prefixed!("pkg/ssr_modes.css")/>
        <Title text="Welcome to Leptos"/>
        <Router>
            <nav>
                <a href=prefixed!("")>"Home"</a>
                // <a href=prefixed!("seminar")>"Seminars"</a>
                // <a href=prefixed!("resources")>"Resource lists"</a>
                <a href=prefixed!("post")>"posts"</a>
                <a href=prefixed!("tabs")>"tabs"</a>
                <a href=prefixed!("csr/a")>"test"</a>
            </nav>
            <main class="content">
                <Routes fallback>
                    <Route
                        path=path!("/")
                        view=HomePage
                        ssr=SsrMode::Static(
                            StaticRoute::new()
                            // .regenerate(|_| watch_path(Path::new("./posts"))),
                        )
                    />

                    <Route
                        path=path!("/about")
                        view=move || view! { <Redirect path="/"/> }
                        ssr=SsrMode::Static(StaticRoute::new())
                    />

                    <Route
                        path=path!("/csr/:param")
                        view=Dynamic
                        ssr=SsrMode::Static(StaticRoute::new()
                        .prerender_params(|| async move {
                            [("param".into(), vec!["a".into(), "b".into()])]
                                .into_iter()
                                .collect()
                        })
                    )/>

                    <ParentRoute path=path!("/post") view=Outlet>
                        <views::posts::Routes/>
                    </ParentRoute>

                    <ParentRoute path=path!("/tabs") view=Outlet>
                        <views::tabs::Routes/>
                    </ParentRoute>

                </Routes>
            </main>
        </Router>
    }
}

#[island]
fn HomePage() -> impl IntoView {
    view! {<Counter/>}
}

#[island]
fn Counter() -> impl IntoView {
    // Creates a reactive value to update the button
    let count = RwSignal::new(0);
    let on_click = move |_| *count.write() += 1;

    view! {
        <button on:click=on_click>"Click Me: " {count}</button>
    }
}


#[component]
fn Dynamic() -> impl IntoView {
    // currently doesn't work as desired because params and queries are weird?
    use leptos_router::hooks;
    use leptos_router::components::Form;

    let params = hooks::use_params_map();
    let queries = hooks::use_query_map();

    let param = move || params.read().get("param");
    let query = move || queries.read().get("q");

    let view_params = move || format!("Params: {:?}", param());
    let view_queries = move || format!("Queries: {:?}", query());

    view!{
        <p> {view_params} </p>
        <p> {view_queries} </p>
        <Form method="GET" action="">
        <input type="text" name="q" value=query/>
        <input type="submit" />
        </Form>

        <p> <Counter/> </p>

        <p> <a href=move || format!("/csr/{}", query().unwrap_or_default())> Go </a> </p>
    }
}

