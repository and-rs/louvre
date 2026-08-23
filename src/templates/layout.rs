use axum::response::Html;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::assets;

use super::{footer, theme_toggle};

pub fn page(title: &str, description: &str, content: Markup) -> Html<String> {
    let document = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                script { (PreEscaped("(function(){var theme=localStorage.getItem('theme')||'system';var dark=theme==='dark'||(theme==='system'&&matchMedia('(prefers-color-scheme: dark)').matches);document.documentElement.classList.toggle('dark',dark);document.documentElement.style.colorScheme=dark?'dark':'light';document.documentElement.dataset.theme=theme})()")) }
                title { (title) " | Sanarte" }
                link rel="apple-touch-icon" sizes="180x180" href=(assets::APPLE_TOUCH_ICON);
                link rel="icon" type="image/png" sizes="32x32" href=(assets::FAVICON_32);
                link rel="icon" type="image/png" sizes="16x16" href=(assets::FAVICON_16);
                link rel="icon" href=(assets::FAVICON_ICO);
                link rel="manifest" href=(assets::FAVICON_MANIFEST);
                link rel="stylesheet" href=(assets::SITE_CSS) data-site-stylesheet data-source=(assets::SITE_CSS);
            }
            body class="min-h-screen pt-20 antialiased sm:pt-24" {
                header class="fixed inset-x-0 top-0 z-50 px-4 pt-3 sm:pt-4" {
                    nav class="mx-auto flex max-w-4xl items-center gap-1 rounded-lg border bg-background/90 p-1 shadow-sm backdrop-blur" aria-label="Main navigation" {
                        a class="inline-flex items-center rounded-md px-2 py-1 transition-colors hover:bg-accent" href="/" {
                            span class="text-base font-semibold tracking-tight" { "Sanarte" }
                        }
                        (theme_toggle())
                    }
                }
                main class="mx-auto w-full max-w-4xl py-10 sm:py-16" { (content) }
                footer class="mx-auto w-full max-w-4xl" {
                    (footer())
                }
                script src=(assets::MU_JS) defer {};
                script src=(assets::SITE_JS) defer {};
                @if cfg!(feature = "dev") {
                    script src="/static/js/dev.js" defer {};
                }
            }
        }
    };

    Html(document.into_string())
}
