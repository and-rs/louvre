use axum::response::Html;
use maud::{DOCTYPE, Markup, html};

pub fn page(title: &str, description: &str, content: Markup) -> Html<String> {
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
                footer { "Axum + Maud + Tailwind + µJS + Anime.js" }
                script src="/static/js/anime.min.js" defer {};
                script src="/static/js/mu.min.js" defer {};
                script src="/static/js/site.js" defer {};
            }
        }
    };

    Html(document.into_string())
}
