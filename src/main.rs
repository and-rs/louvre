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
            h1 { "Rust site baseline" }
            p { "Server-rendered HTML, Markdown, µJS navigation, and browser JavaScript." }
            p { a href="/articles" { "Read the sample article" } " | " a href="#animation" { "See animation" } }
            section id="animation" data-orbit-demo {
                h2 { "Anime.js" }
                svg viewBox="0 0 240 240" width="240" height="240" {
                    circle cx="120" cy="120" r="90" fill="none" stroke="currentColor" {}
                    circle cx="120" cy="120" r="10" fill="currentColor" {}
                    rect x="112" y="20" width="16" height="16" data-orbit-node {}
                }
                p { "This runs after a normal page load and µJS navigation." }
            }
            section data-performance-metrics {
                h2 { "Browser performance" }
                dl {
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
            section {
                h1 { "Articles" }
                article {
                    p { (&article.published_at) }
                    h2 { a href=(format!("/articles/{}", article.slug)) { (&article.title) } }
                    p { (&article.description) }
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
            article {
                p { a href="/articles" { "<- All articles" } }
                p { (&article.published_at) }
                h1 { (&article.title) }
                p { (&article.description) }
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
                section {
                    h1 { "404" }
                    p { "That page does not exist." }
                    p { a href="/" { "Return home" } }
                }
            },
        ),
    )
        .into_response()
}

fn page(title: &str, description: &str, content: Markup) -> Html<String> {
    let document = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                title { (title) " | Rust Site" }
                link rel="stylesheet" href="/static/css/site.css";
            }
            body {
                header {
                    nav aria-label="Main navigation" {
                        a href="/" { "Rust site" }
                        " | "
                        a href="/articles" { "Articles" }
                    }
                }
                main { (content) }
                footer {
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
        div data-metric=(key) {
            dt { (label) }
            dd { (description) }
            dd data-metric-value { "..." }
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
