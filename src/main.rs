use std::{net::SocketAddr, sync::Arc};

use axum::{
    Router,
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Response},
    routing::get,
};
use maud::{DOCTYPE, Markup, html};
use pulldown_cmark::{Options, Parser, html::push_html};
use serde::Deserialize;
use thiserror::Error;
use tokio::net::TcpListener;
use tower_http::{services::ServeDir, trace::TraceLayer};
#[cfg(feature = "dev")]
use tower_livereload::LiveReloadLayer;

const ARTICLE_SOURCE: &str = include_str!("../content/articles/faster-by-default.md");

#[derive(Clone)]
struct AppState {
    article: Article,
}

#[derive(Clone)]
struct Article {
    slug: &'static str,
    title: String,
    description: String,
    published_at: String,
    html: String,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    description: String,
    published_at: String,
}

#[derive(Debug, Error)]
enum ArticleError {
    #[error("article is missing a closing frontmatter delimiter")]
    MissingFrontmatter,
    #[error("invalid article frontmatter: {0}")]
    InvalidFrontmatter(#[from] serde_yaml::Error),
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let article = parse_article("faster-by-default", ARTICLE_SOURCE)
        .expect("embedded article must have valid frontmatter");
    let state = Arc::new(AppState { article });

    let app = Router::new()
        .route("/", get(home))
        .route("/articles", get(articles))
        .route("/articles/{slug}", get(article_page))
        .fallback(not_found)
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    #[cfg(feature = "dev")]
    let app = app.layer(LiveReloadLayer::new());

    let address = SocketAddr::from(([127, 0, 0, 1], 3000));
    let listener = TcpListener::bind(address)
        .await
        .expect("local port 3000 must be available");
    tracing::info!(%address, "listening");
    axum::serve(listener, app).await.expect("server failed");
}

async fn home() -> Html<String> {
    page(
        "Rust Site Baseline",
        "A server-rendered proof of concept.",
        html! {
            section class="grid gap-6 lg:grid-cols-[1.2fr_.8fr]" {
                div class="space-y-5" {
                    p class="font-mono text-sm tracking-wide text-cyan-300" { "AXUM / MAUD / TAILWIND" }
                    h1 class="max-w-3xl text-5xl font-semibold tracking-tight text-white sm:text-7xl" {
                        "Server HTML. "
                        span class="text-cyan-300" { "No hydration." }
                    }
                    p class="max-w-xl text-lg leading-8 text-slate-300" {
                        "This is a deliberately small baseline for testing Rust-rendered pages, progressive navigation, Markdown articles, and imperative animation."
                    }
                    div class="flex flex-wrap gap-3" {
                        a class="button button-primary" href="/articles" { "Read the sample article" }
                        a class="button" href="#animation" { "See animation" }
                    }
                }
                section id="animation" class="panel grid min-h-80 place-items-center overflow-hidden" data-orbit-demo {
                    div class="relative size-52" {
                        div class="absolute inset-0 rounded-full border border-cyan-300/30" {}
                        div class="absolute left-1/2 top-1/2 size-5 -translate-x-1/2 -translate-y-1/2 rounded-full bg-cyan-300 shadow-[0_0_40px_rgba(103,232,249,.75)]" {}
                        div class="absolute left-1/2 top-0 size-6 -translate-x-1/2 rounded-sm bg-violet-400" data-orbit-node {}
                    }
                    p class="col-span-full text-sm text-slate-400" { "Anime.js is initialized after full and µJS navigation." }
                }
            }
            section class="panel mt-6" data-performance-metrics {
                div class="mb-5 flex items-baseline justify-between gap-4 border-b border-slate-800 pb-4" {
                    h2 class="text-xl font-semibold text-white" { "Browser performance" }
                    span class="font-mono text-xs text-slate-500" { "THIS PAGE" }
                }
                div class="grid gap-x-8 gap-y-4 sm:grid-cols-2 lg:grid-cols-3" {
                    (metric("TTFB", "Time to First Byte", "Server responsiveness"))
                    (metric("DOM_READY", "DOM Ready", "Document structure parsed"))
                    (metric("FP", "First Paint", "Visual feedback started"))
                    (metric("FCP", "First Contentful Paint", "Content begins appearing"))
                    (metric("LCP", "Largest Contentful Paint", "Main content feels ready"))
                    (metric("CLS", "Cumulative Layout Shift", "Visual layout is stable"))
                }
            }
        },
    )
}

async fn articles(State(state): State<Arc<AppState>>) -> Html<String> {
    let article = &state.article;
    page(
        "Articles",
        "Markdown rendered by Rust.",
        html! {
            section class="space-y-6" {
                div class="space-y-2" {
                    p class="font-mono text-sm tracking-wide text-cyan-300" { "ARTICLES" }
                    h1 class="text-5xl font-semibold tracking-tight text-white" { "Writing from the server." }
                }
                a class="panel block space-y-3 transition hover:-translate-y-1 hover:border-cyan-300/70" href=(format!("/articles/{}", article.slug)) {
                    p class="font-mono text-sm text-slate-400" { (&article.published_at) }
                    h2 class="text-2xl font-semibold text-white" { (&article.title) }
                    p class="text-slate-300" { (&article.description) }
                }
            }
        },
    )
}

async fn article_page(Path(slug): Path<String>, State(state): State<Arc<AppState>>) -> Response {
    let article = &state.article;
    if slug != article.slug {
        return not_found().await;
    }

    page(
        &article.title,
        &article.description,
        html! {
            article class="prose prose-invert max-w-3xl" {
                a class="font-mono text-sm text-cyan-300 hover:text-cyan-100" href="/articles" { "<- All articles" }
                p class="mt-8 font-mono text-sm text-slate-400" { (&article.published_at) }
                h1 { (&article.title) }
                p class="lead" { (&article.description) }
                (maud::PreEscaped(&article.html))
            }
        },
    )
    .into_response()
}

async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        page(
            "Not found",
            "The requested page does not exist.",
            html! {
                section class="panel max-w-xl space-y-4" {
                    p class="font-mono text-sm text-cyan-300" { "404" }
                    h1 class="text-4xl font-semibold text-white" { "That page does not exist." }
                    a class="button button-primary" href="/" { "Return home" }
                }
            },
        ),
    )
        .into_response()
}

fn page(title: &str, description: &str, content: Markup) -> Html<String> {
    let document = html! {
        (DOCTYPE)
        html lang="en" class="bg-slate-950" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                title { (title) " | Rust Site" }
                link rel="stylesheet" href="/static/css/site.css";
            }
            body class="min-h-screen bg-slate-950 text-slate-100 antialiased" {
                header class="border-b border-slate-800" {
                    nav class="mx-auto flex max-w-6xl items-center justify-between px-6 py-5" aria-label="Main navigation" {
                        a class="font-mono text-sm font-bold tracking-widest text-cyan-300" href="/" { "RUST SITE" }
                        div class="flex gap-5 text-sm text-slate-300" {
                            a class="hover:text-cyan-300" href="/" { "Home" }
                            a class="hover:text-cyan-300" href="/articles" { "Articles" }
                        }
                    }
                }
                main class="mx-auto max-w-6xl px-6 py-16" { (content) }
                footer class="mx-auto max-w-6xl px-6 pb-10 text-sm text-slate-500" {
                    "Axum + Maud + Tailwind + µJS + Anime.js"
                }
                script src="/static/js/anime.min.js" defer {};
                script src="/static/js/mu.min.js" defer {};
                script src="/static/js/site.js" defer {};
            }
        }
    };

    Html(document.into_string())
}

fn metric(key: &str, label: &str, description: &str) -> Markup {
    html! {
        div class="border-b border-slate-800 pb-3" data-metric=(key) {
            p class="font-medium text-slate-100" { (label) }
            p class="text-sm text-slate-400" { (description) }
            p class="mt-2 font-mono text-2xl text-cyan-300" data-metric-value { "..." }
        }
    }
}

fn parse_article(slug: &'static str, source: &str) -> Result<Article, ArticleError> {
    let source = source.trim();
    let frontmatter = source
        .strip_prefix("---\n")
        .ok_or(ArticleError::MissingFrontmatter)?;
    let (frontmatter, body) = frontmatter
        .split_once("\n---\n")
        .ok_or(ArticleError::MissingFrontmatter)?;
    let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter)?;
    let mut html = String::new();
    let parser = Parser::new_ext(body, Options::all());
    push_html(&mut html, parser);

    Ok(Article {
        slug,
        title: frontmatter.title,
        description: frontmatter.description,
        published_at: frontmatter.published_at,
        html,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_markdown() {
        let article = parse_article(
            "example",
            "---\ntitle: Example\ndescription: A test article.\npublished_at: 2026-08-06\n---\n\n# Hello",
        )
        .unwrap();

        assert_eq!(article.title, "Example");
        assert!(article.html.contains("<h1>Hello</h1>"));
    }
}
