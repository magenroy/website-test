pub mod app;
pub mod views;

use leptos::prelude::*;
use std::path::{PathBuf, Path};
use futures::{channel::mpsc, Stream};

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
    ($path:tt) => {format!("{}/{}", prefix(), $path)}
}

pub mod prelude {
    pub use leptos::prelude::*;
    pub use super::prefixed;
    pub use super::prefix;
    pub use super::with_prefix;
    pub use super::list_server_slugs;
    pub use super::list_slugs;
}


#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}

// use leptos::prelude::*;
// use std::path::Path;
// // #[server] don't make this a server function -- just use it inside server functions!
// pub async fn list_slugs(path: impl AsRef<Path>, extension: &str) -> Result<Vec<String>, ServerFnError> {
//     use tokio::fs;
//     use tokio_stream::wrappers::ReadDirStream;
//     use tokio_stream::StreamExt;
//
//     let files = ReadDirStream::new(fs::read_dir(path).await?);
//     Ok(files
//         .filter_map(|entry| {
//             let entry = entry.ok()?;
//             let path = entry.path();
//             if !path.is_file() {
//                 return None;
//             }
//             if path.extension()? != extension {
//                 return None;
//             }
//
//             let slug = path
//                 .file_name()
//                 .and_then(|n| n.to_str())
//                 .unwrap_or_default()
//                 .replace(extension, "");
//             Some(slug)
//         })
//         .collect()
//         .await)
// }
//
//
#[server]
pub async fn list_server_slugs(path: PathBuf, extension: String) -> Result<Vec<String>, ServerFnError> {
    use tokio::fs;
    use tokio_stream::wrappers::ReadDirStream;
    use tokio_stream::StreamExt;

    // I think this should only get run after server generates stuff?
    // let path = {
    //     let mut tmp = PathBuf::new();
    //     // tmp.push(PREFIX);
    //     tmp.push("/pkg/");
    //     tmp.extend(path.iter());
    //     tmp
    // };

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

pub fn list_slugs(path: impl AsRef<Path>, extension: &str) -> Result<Vec<String>> {
    use std::fs;

    // I think this should only get run after server generates stuff?
    // let path = {
    //     let mut tmp = PathBuf::new();
    //     // tmp.push(PREFIX);
    //     tmp.push("/pkg/");
    //     tmp.extend(path.iter());
    //     tmp
    // };

    let files = fs::read_dir(with_prefix(&path))?;
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

            let slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .replace(&extension, "");
            Some(slug)
        })
        .collect())
}

#[allow(unused)] // path is not used in non-SSR
fn watch_path(path: &Path) -> impl Stream<Item = ()> {
    #[allow(unused)]
    let (mut tx, rx) = mpsc::channel(0);

    #[cfg(feature = "ssr")]
    {
        use notify::RecursiveMode;
        use notify::Watcher;

        let mut watcher =
            notify::recommended_watcher(move |res: Result<_, _>| {
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
        watcher
            .watch(path, RecursiveMode::NonRecursive)
            .expect("could not watch path");

        // we want this to run as long as the server is alive
        std::mem::forget(watcher);
    }

    rx
}
