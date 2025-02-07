pub mod app;
pub mod views;

use leptos::prelude::*;
use std::path::{PathBuf, Path};
use futures::{channel::mpsc, Stream};

// const PREFIX: &str = "website-test";
pub fn prefix() -> String {
    std::env::var("PREFIX").unwrap_or("".into())
}

pub fn with_prefix(path: &str) -> String {
    format!("/{}/{path}", prefix())
}

#[macro_export]
macro_rules! prefixed {
    ($path:tt) => {format!("/{}/{}", prefix(), $path)}
}


/* #[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
} */

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
pub async fn list_slugs(path: PathBuf, extension: String) -> Result<Vec<String>, ServerFnError> {
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
