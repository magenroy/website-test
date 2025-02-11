pub mod app;
pub mod views;

use futures::{channel::mpsc, Stream};
use leptos::{children, prelude::*};
use leptos_router::{ChooseView, MatchNestedRoutes, PossibleRouteMatch};
use std::{
    fs::DirEntry, future::Future, path::{Path, PathBuf}
};

// const PREFIX: &str = "/website-test";
pub fn prefix() -> String {
    if let Ok(prefix) = std::env::var("PREFIX") {
        format!("/{}", prefix)
    } else {
        Default::default()
    }
}

pub fn with_prefix(path: impl AsRef<Path>) -> PathBuf {
    // format!("{}/{path}", prefix())
    let mut out = PathBuf::from(prefix());
    out.push(path);
    out
}

#[macro_export]
macro_rules! prefixed {
    ($path:tt) => {
        format!("{}/{}", prefix(), $path)
    };
}

pub mod prelude {
    pub use super::prefix;
    pub use super::prefixed;
    pub use super::watch_path;
    pub use super::with_prefix;

    pub use super::list_slugs;
    pub use super::ParamRoute;

    pub use leptos::prelude::*;
}

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    // leptos::mount::hydrate_body(app::App);
    leptos::mount::hydrate_islands();
}

// #[component(transparent)]
// pub fn ParamRoute(param_name: &'static str, view: impl ChooseView) -> impl MatchNestedRoutes + Clone {
//     use leptos_router::components::Route;
//     view! {
//         <Route path=(leptos_router::ParamSegment(param_name),) view/>
//     }.into_inner()
// }

#[server]
pub async fn scan_dir(path: String) -> Result<Vec<PathBuf>, ServerFnError> {
    use tokio::fs;
    use tokio_stream::wrappers::ReadDirStream;
    use tokio_stream::StreamExt;

    println!("READING {path}");
    let files = ReadDirStream::new(fs::read_dir(&path).await?);

    // let files = std::fs::read_dir(&path).unwrap();

    let out = files.filter_map(
        |d| {
            println!("{:?}", d);
            Some(d.ok()?.path())
            // let p = d.ok()?.path();
            // Some(p.to_str()?.to_string())
            // // if p.is_file() {
            // //     Some(format!("asdf/{}",p.file_stem()?.to_str()?))
            // // } else { None }
        }
    ).collect();

    Ok(out.await)
}

#[component(transparent)]
pub fn ParamRoute(
    #[prop(default="slug")] param_name: &'static str,
    view: impl ChooseView,
    dir_path: &'static str,
    // By default just reads all files in the directory, and outputs the stems
    #[prop(optional)] mapper: Option<Box<dyn Fn(DirEntry) -> Option<String>>>,
) -> impl MatchNestedRoutes + Clone {
    use leptos_router::components::*;
    use leptos_router::{path, ParamSegment, SsrMode, static_routes::{StaticRoute, StaticParamsMap}};

    /* let default = |e: tokio::fs::DirEntry| -> Option<String> {
        let p = e.path();
        if p.is_file() {
            Some(format!("asdf/{}",p.file_stem()?.to_str()?))
        } else { None }
    }; */
    // let slugs = match &mapper {
    //     _ => scan_dir(&dir_path, default),
    //     // Some(f) => scan_dir(&dir_path, f)
    // };
    let slugs: Vec<String> = std::fs::read_dir(&dir_path).unwrap().
        filter_map(|d| {
            let out = match &mapper {
                Some(f) => f(d.ok()?),
                None => {
                    let p = d.ok()?.path();
                    if !p.is_file() { return None }
                    Some(p.file_stem()?.to_str()?.to_string())
                }
            };
            println!("{:?}", out);
            out
        }).collect();

    println!("num of paths: {}", slugs.len());

    view! {
        <Route path=path!("/") view=move || view!{qwerty}/>
        <Route
            // path=(ParamSegment(param_name),)
            path=path!("/:slug")
            view=view
            ssr=SsrMode::Static(
                StaticRoute::new()
                    .prerender_params(move || {
        // need to take ownership of this variables before moving into async block
        // and need to define slugs before the closure body
        let slugs = slugs.clone();
        async move {
            // let slugs: Vec<String> = scan_dir(dir_path.to_string()).await.unwrap().into_iter().map(|p: PathBuf| -> String {p.to_str().unwrap().to_string()}).collect();
            // let mut params = StaticParamsMap::new();
            // params.insert(param_name, slugs);
            // params
            [("slug".into(), slugs)].into_iter().collect()
    }})
            )
        />
    }
    .into_inner();
}

#[server]
pub async fn list_slugs(path: PathBuf, extension: String) -> Result<Vec<String>, ServerFnError> {
    use tokio::fs;
    use tokio_stream::wrappers::ReadDirStream;
    use tokio_stream::StreamExt;

    let files = ReadDirStream::new(fs::read_dir(&path).await?);
    Ok(files
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            if path.extension()? != std::ffi::OsStr::new(&extension) {
                return None;
            }

            let extension = format!(".{extension}");
            let slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .replace(&extension, "");
            Some(slug)
        })
        .collect()
        .await)
}

// pub fn read_file<F,O,E>(path: impl AsRef<Path>, f: F) -> Result<Option<O>, E> where F: FnOnce(&str) -> Result<Option<O>, E> { }

#[allow(unused)] // path is not used in non-SSR
pub fn watch_path(path: impl AsRef<Path>) -> impl Stream<Item = ()> {
    #[allow(unused)]
    let (mut tx, rx) = mpsc::channel(0);

    #[cfg(feature = "ssr")]
    {
        use notify::RecursiveMode;
        use notify::Watcher;

        let mut watcher = notify::recommended_watcher(move |res: Result<_, _>| {
            if res.is_ok() {
                // if this fails, it's because the buffer is full
                // this means we've already notified before it's regenerated,
                // so this page will be queued for regeneration already
                _ = tx.try_send(());
            }
        })
        .expect("could not create watcher");

        // Add a path to be watched. All files and directories at that path and
        // below will be monitored for changes.
        if watcher
            .watch(path.as_ref(), RecursiveMode::NonRecursive)
            .is_err()
        {
            leptos::logging::log!("could not watch path")
        }
        // .expect("could not watch path");

        // we want this to run as long as the server is alive
        std::mem::forget(watcher);
    }

    rx
}
