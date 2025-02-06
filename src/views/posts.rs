use crate::list_slugs;
use std::path::PathBuf;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, A},
    hooks::use_params,
    params::Params,
    path,
    MatchNestedRoutes,
    static_routes::StaticRoute,
    SsrMode,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;


// I think that in order to get it to generate the routes for all the slugs
// I need to modify
// probably put all the posts into assets, and generate routes using assets?
// Or maybe make those routes be done with CSR instead of SSR?


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
                path=path!(":slug")
                view=Post
                ssr=SsrMode::Static(
                    StaticRoute::new()
                        .prerender_params(|| async move {
                            [("slug".into(), list_slugs(PathBuf::from("./posts"), String::from(".md")).await.unwrap_or_default())]
                                .into_iter()
                                .collect()
                        })
                        /* .regenerate(|params| {
                            let slug = params.get("slug").unwrap();
                            watch_path(Path::new(&format!("./posts/{slug}.md")))
                        }), */
                )
            />
    }.into_inner()
}

#[component]
pub fn HomePage() -> impl IntoView {
    // load the posts
    let posts = Resource::new(|| (), |_| list_posts());
    let posts = move || {
        posts
            .get()
            .map(|n| n.unwrap_or_default())
            .unwrap_or_default()
    };

    view! {
        <h1>"My Great Blog"</h1>
        <Suspense fallback=move || view! { <p>"Loading posts..."</p> }>
            <ul>
                <For each=posts key=|post| post.slug.clone() let:post>
                    <li>
                        <A href=post.slug>{post.title.clone()}</A>
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
pub fn Post() -> impl IntoView {
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
                Ok(Err(e)) | Err(e) => {
                    Err(PostError::ServerError(e.to_string()))
                }
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
    slug: String,
    title: String,
    content: String,
}


#[server]
pub async fn list_posts() -> Result<Vec<Post>, ServerFnError> {
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

            Ok(Some(Post {
                slug,
                title,
                content,
            }))
        })
        .try_collect()
        .await
        .map_err(ServerFnError::from)
}

#[server]
pub async fn get_post(slug: String) -> Result<Option<Post>, ServerFnError> {
    println!("reading ./posts/{slug}.md");
    let content =
        tokio::fs::read_to_string(&format!("./posts/{slug}.md")).await?;
    // world's worst Markdown frontmatter parser
    let title = content.lines().next().unwrap().replace("# ", "");

    Ok(Some(Post {
        slug,
        title,
        content,
    }))
}
