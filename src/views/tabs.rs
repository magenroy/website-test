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
                view=Test
                ssr=SsrMode::Static(
                    StaticRoute::new()
                        /* .prerender_params(|| async move {
                            // use leptos_router::static_routes::*;
                            // let mut params = StaticParamsMap::new();
                            // params.insert(
                            //     "slug",
                            //     list_slugs( PathBuf::from("/posts"), String::from(".md"))
                            //         .await.unwrap_or_default()
                            // );
                            // params
                            [("slug".into(), list_slugs("./posts".into(), "md".into()).await.unwrap_or_default())]
                                .into_iter()
                                .collect()
                        }) */
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
fn Test() -> impl IntoView {
    let param = leptos_router::hooks::use_params_map().read().get("slug").unwrap_or_default();

    let path = format!("./posts/{param}.md");

    // Since I intend to just use the server to produce static pages, I can just read the file
    // (synchronously!) on the server in order to produce the page
    let content = std::fs::read_to_string(path);

    view! {
        {content}
    }
}

#[island]
fn Tabs(labels: Vec<String>, children: Children) -> impl IntoView {
    let (selected, set_selected) = signal(1);
    provide_context(selected);

    let buttons = labels
        .into_iter()
        .enumerate()
        .map(|(index, label)| view! { <button on:click=move |_| set_selected.set(index)>{label}</button> })
        .collect_view();
    view! {
        <div style="display: flex; width: 100%; justify-content: space-around;">
            {buttons}
        </div>
        {children()}
    }
}
#[island]
fn Tab(index: usize, children: Children) -> impl IntoView {
    let selected = expect_context::<ReadSignal<usize>>();
    view! {
        <div
            style:background-color="lightblue"
            style:padding="10px"
            style:display=move || if selected.get() == index { "block" } else { "none" }
        >
            {children()}
        </div>
    }
}

#[island]
fn LocalTab(paths: Vec<String>, children: Children) -> impl IntoView {
    let labels = paths.clone();

    let paths: Vec<PathBuf> = paths.into_iter().map(with_prefix).collect();

    let selected = expect_context::<ReadSignal<usize>>();

    view! {
        {move || labels[selected.get()].clone()}: 
        {move || std::fs::read_to_string(paths[selected.get()].clone())}
        <div> {children()} </div>
    }

}

#[component]
fn HomePage() -> impl IntoView {
    // NOTE: this is not so good
    // since we are actually reading all the files in order to display this page
    // even though we only want to show one of them
    // but this example serves to show that since we are in "islands mode", and this is a
    // `component`, this code only runs on the server!
    let files = ["./posts/post1.md", "./posts/post4.md", "./posts/post3.md"];
    let labels: Vec<String> = files.iter().copied().map(Into::into).collect();
    let tabs = move || {
        files
            .into_iter()
            .enumerate()
            .map(|(index, filename)| {
                let content = std::fs::read_to_string(filename).unwrap();
                view! {
                    <Tab index>
                        <h2>{filename.to_string()}</h2>
                        <p>{content}</p>
                    </Tab>
                }
            })
            .collect_view()
    };

    view! {
        <h1>"Welcome to Leptos!"</h1>
        <p>"Click any of the tabs below to read a recipe."</p>
        <Tabs labels={labels.clone()}>
            <div>{tabs()}</div>
            <div> <LocalTab paths=labels> asdf </LocalTab> </div>
        </Tabs>
    }
}
