use crate::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, A},
    hooks::use_params,
    params::Params,
    path,
    static_routes::StaticRoute,
    MatchNestedRoutes, SsrMode,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

// I think that in order to get it to generate the routes for all the slugs
// I need to modify
// probably put all the posts into assets, and generate routes using assets?
// Or maybe make those routes be done with CSR instead of SSR?
//
// Put into assets. Need to replace the server-side functions that look for files with client-side ones

#[component(transparent)]
pub fn PostRoutes() -> impl MatchNestedRoutes + Clone {
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
                view=PostView

                ssr=SsrMode::Static(
                    StaticRoute::new()
                        .prerender_params(|| async move {
                            // use leptos_router::static_routes::*;
                            // let mut params = StaticParamsMap::new();
                            // params.insert(
                            //     "slug",
                            //     list_slugs( PathBuf::from("/posts"), String::from(".md"))
                            //         .await.unwrap_or_default()
                            // );
                            // params
                            [("slug".into(), list_slugs(PathBuf::from("./posts"), String::from(".md")).await.unwrap_or_default())]
                            // [("slug".into(), list_slugs("/posts", ".md").unwrap_or_default())]
                                .into_iter()
                                .collect()
                        })
                        .regenerate(|params| {
                            let slug = params.get("slug").unwrap();
                            watch_path(std::path::Path::new(&format!("./posts/{slug}.md")))
                        }),
                )
            />
    }
    .into_inner()
}

#[component]
pub fn HomePage() -> impl IntoView {
    // load the posts
    // NOTE: need to list_posts on the server, since we only generate the static pages while the
    // server is running

    StaticRoute::new().prerender_params(|| async move {
        use leptos_router::static_routes::*;
        let mut params = StaticParamsMap::new();
        params.insert(
            "slug",
            list_slugs(PathBuf::from("/posts"), String::from(".md"))
                .await
                .unwrap_or_default(),
        );
        params
    });
    let posts = Resource::new(|| (), |_| list_posts());
    let posts = move || {
        posts
            .get()
            .map(|n| n.unwrap_or_default())
            .unwrap_or_default()
    };

    let addr = |slug: &str| -> String {
        let relative = format!("post/{}", slug);
        prefixed!(relative)
    };

    view! {
        <h1>"My Great Blog"</h1>
        <Suspense fallback=move || view! { <p>"Loading posts..."</p> }>
            <ul>
                // {list_posts_client().unwrap_or_default() .into_iter()
                //     .map(|(slug,post)| view!{
                //         <li>
                //             <a href=addr(&slug)>{post.title}</a>
                //         </li>
                //     }).collect::<Vec<_>>()
                // }
                <For each=posts key=|(slug, _)| slug.clone() let:((slug,post))>
                    <li>
                        <a href=addr(&slug)>{post.title.clone()}</a>
                    </li>
                </For>
            </ul>
        </Suspense>
    }
}

#[derive(Params, Clone, Debug, PartialEq, Eq)]
struct PostParams {
    slug: Option<String>,
}

#[component]
pub fn PostView() -> impl IntoView {
    //     let slug = use_params::<PostParams>().get().unwrap().slug.unwrap();
    //     Post(slug)
    // }
    //
    // pub fn Post(slug: String) -> impl IntoView {
    let query = use_params::<PostParams>();
    let slug = move || {
        query
            .get()
            .map(|q| q.slug.unwrap_or_default())
            .map_err(|_| PostError::InvalidId)
    };
    let post_resource = Resource::new_blocking(slug, |slug| async move {
        match slug {
            Err(e) => Err(e),
            Ok(slug) => get_post(slug)
                .await
                .map(|data| data.ok_or(PostError::PostNotFound))
                .map_err(|e| PostError::ServerError(e.to_string())),
        }
    });

    // let slug = query.get().unwrap_or({return view!{"asdf"}}).slug.unwrap_or({return view!{"qwer"}});;
    // let slug = query.get().unwrap().slug.unwrap();

    // match read_post(&slug) {
    //     Err(_) => view!{<p> "Error" </p>}.into_any(),
    //     Ok(None) => view!{<p> "Error: post not found" </p>}.into_any(),
    //     Ok(Some(post)) => view!{
    //         <em>"The world's best content."</em>
    //         <h1>{post.title.clone()}</h1>
    //         <p>{post.content.clone()}</p>
    //
    //         // since we're using async rendering for this page,
    //         // this metadata should be included in the actual HTML <head>
    //         // when it's first served
    //         <Title text=post.title/>
    //         <Meta name="description" content=post.content/>
    //     }.into_any(),
    // }
    /* let post_view = read_post(&slug).map(|x| x.map(|post| view! {
            <em>"The world's best content."</em>
            <h1>{post.title.clone()}</h1>
            <p>{post.content.clone()}</p>

            // since we're using async rendering for this page,
            // this metadata should be included in the actual HTML <head>
            // when it's first served
            <Title text=post.title/>
            <Meta name="description" content=post.content/>
    }));

    view! {
        <ErrorBoundary fallback=|errors| {
            #[cfg(feature = "ssr")]
            expect_context::<leptos_axum::ResponseOptions>()
                .set_status(http::StatusCode::NOT_FOUND);
            view! {
                <div class="error">
                    <h1>"Something went wrong."</h1>
                    <ul>
                        {move || {
                            errors
                                .get()
                                .into_iter()
                                .map(|(_, error)| view! { <li>{error.to_string()}</li> })
                                .collect::<Vec<_>>()
                        }}

                    </ul>
                </div>
            }
        }>{post_view}</ErrorBoundary>
    } */

    let post_view = move || {
        Suspend::new(async move {
            match post_resource.await {
                Ok(Ok(post)) => {
                    Ok(view! {
                        <h1>{post.title.clone()}</h1>
                        <p>{post.content.clone()}</p>

                        // since we're using async rendering for this page,
                        // this metadata should be included in the actual HTML <head>
                        // when it's first served
                        <Title text=post.title/>
                        <Meta name="description" content=post.content/>
                    })
                }
                Ok(Err(e)) | Err(e) => Err(PostError::ServerError(e.to_string())),
            }
        })
    };

    view! {
        <em>"The world's best content."</em>
        <Suspense fallback=move || view! { <p>"Loading post..."</p> }>
            <ErrorBoundary fallback=|errors| {
                #[cfg(feature = "ssr")]
                expect_context::<leptos_axum::ResponseOptions>()
                    .set_status(http::StatusCode::NOT_FOUND);
                view! {
                    <div class="error">
                        <h1>"Something went wrong."</h1>
                        <ul>
                            {move || {
                                errors
                                    .get()
                                    .into_iter()
                                    .map(|(_, error)| view! { <li>{error.to_string()}</li> })
                                    .collect::<Vec<_>>()
                            }}

                        </ul>
                    </div>
                }
            }>{post_view}</ErrorBoundary>
        </Suspense>
    }
}

#[derive(Error, Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PostError {
    #[error("Invalid post ID.")]
    InvalidId,
    #[error("Post not found.")]
    PostNotFound,
    #[error("Server error: {0}.")]
    ServerError(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Post {
    // slug: String,
    title: String,
    content: String,
}

pub fn list_posts_client() -> Result<Vec<(String, Post)>> {
    println!("calling list_posts");

    let path = "posts";
    let extension = ".md";

    let files = std::fs::read_dir(prefixed!(path))?;
    Ok(files
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let path = entry.path();
            if !path.is_file() {
                return None;
            }
            if path.extension()? != std::ffi::OsStr::new(extension) {
                return None;
            }

            let slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .replace(".md", "");

            let content = std::fs::read_to_string(path).ok()?;
            // world's worst Markdown frontmatter parser
            // let title = content.lines().next().unwrap().replace("# ", "");
            let title = slug.clone();

            Some((
                slug,
                Post {
                    // slug,
                    title,
                    content,
                },
            ))
        })
        .collect())
}

#[server]
pub async fn list_posts() -> Result<Vec<(String, Post)>, ServerFnError> {
    println!("calling list_posts");

    use futures::TryStreamExt;
    use tokio::fs;
    use tokio_stream::wrappers::ReadDirStream;

    let files = ReadDirStream::new(fs::read_dir("./posts").await?);
    files
        .try_filter_map(|entry| async move {
            let path = entry.path();
            if !path.is_file() {
                return Ok(None);
            }
            let Some(extension) = path.extension() else {
                return Ok(None);
            };
            if extension != "md" {
                return Ok(None);
            }

            let slug = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .replace(".md", "");
            let content = fs::read_to_string(path).await?;
            // world's worst Markdown frontmatter parser
            let title = content.lines().next().unwrap().replace("# ", "");

            Ok(Some((
                slug,
                Post {
                    // slug,
                    title,
                    content,
                },
            )))
        })
        .try_collect()
        .await
        .map_err(ServerFnError::from)
}

fn read_post(slug: &str) -> Result<Option<Post>> {
    let path = with_prefix(format!("posts/{slug}.md"));
    println!("reading {}", path.as_os_str().to_str().unwrap_or("???"));

    let content = std::fs::read_to_string(path)?;

    // world's worst Markdown frontmatter parser
    // let title = content.lines().next().unwrap().replace("# ", "");
    let title = String::from(slug);

    Ok(Some(Post {
        // slug,
        title,
        content,
    }))
}

#[server]
pub async fn get_post(slug: String) -> Result<Option<Post>, ServerFnError> {
    println!("reading ./posts/{slug}.md");
    let content = std::fs::read_to_string(with_prefix(format!("posts/{slug}.md")))?;
    // tokio::fs::read_to_string(&format!("./posts/{slug}.md")).await?;
    // world's worst Markdown frontmatter parser
    let title = content.lines().next().unwrap().replace("# ", "");

    Ok(Some(Post {
        // slug,
        title,
        content,
    }))
}
