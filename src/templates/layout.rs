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
                link rel="apple-touch-icon" sizes="180x180" href="/static/favicon/apple-touch-icon.png";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon/favicon-16x16.png";
                link rel="icon" href="/static/favicon/favicon.ico";
                link rel="manifest" href="/static/favicon/site.webmanifest";
                link rel="stylesheet" href="/static/css/site.css";
            }
            body {
                header {
                    nav aria-label="Main navigation" {
                        a href="/" { "Home" }
                        " | "
                        a href="/work" { "Work" }
                        " | "
                        a href="/articles" { "Articles" }
                    }
                }
                main class="max-w-800px mx-auto my-4" { (content) }
                footer {
                    p { "Researching the future of performance." }
                    p { "Copyright (c) 2026 Iridium. All Rights Reserved." }
                    p {
                        a href="https://www.linkedin.com/company/iridium-tech" target="_blank" rel="noreferrer" { "LinkedIn" }
                        " | "
                        a href="https://www.github.com/and-rs/iridium" target="_blank" rel="noreferrer" { "GitHub" }
                    }
                }
                script src="/static/js/anime.min.js" defer {};
                script src="/static/js/mu.min.js" defer {};
                script src="/static/js/site.js" defer {};
            }
        }
    };

    Html(document.into_string())
}
