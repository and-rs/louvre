use axum::response::Html;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use crate::assets;

use super::{ButtonSize, ButtonVariant, LinkTarget, button_link, footer, theme_toggle};

pub struct PageMetadata<'a> {
    pub page_title: Option<&'a str>,
    pub description: &'a str,
}

pub fn page(metadata: PageMetadata<'_>, content: Markup) -> Html<String> {
    let document = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(metadata.description);
                script { (PreEscaped("(function(){var theme=localStorage.getItem('theme')||'system';var dark=theme==='dark'||(theme==='system'&&matchMedia('(prefers-color-scheme: dark)').matches);document.documentElement.classList.toggle('dark',dark);document.documentElement.style.colorScheme=dark?'dark':'light';document.documentElement.dataset.theme=theme})()")) }
                title {
                    @if let Some(page_title) = metadata.page_title {
                        (page_title) " | "
                    }
                    "Louvre"
                }
                link rel="apple-touch-icon" sizes="180x180" href=(assets::APPLE_TOUCH_ICON);
                link rel="icon" type="image/png" sizes="32x32" href=(assets::FAVICON_32);
                link rel="icon" type="image/png" sizes="16x16" href=(assets::FAVICON_16);
                link rel="icon" href=(assets::FAVICON_ICO);
                link rel="manifest" href=(assets::FAVICON_MANIFEST);
                link rel="stylesheet" href=(assets::SITE_CSS);
            }
            body class="min-h-screen pt-20 antialiased sm:pt-24" {
                header class="fixed inset-x-0 top-0 z-50" {
                    div class="pt-3 sm:pt-4 page-shell" {
                        nav class="flex items-center gap-1 rounded-lg border bg-background/90 p-1 shadow-sm backdrop-blur" aria-label="Main navigation" {
                            (button_link("Louvre", "/", LinkTarget::CurrentTab, ButtonVariant::Ghost, ButtonSize::Default, None))
                            (theme_toggle())
                        }
                    }
                }
                main class="py-10 sm:py-16 page-shell" { (content) }
                footer class="page-shell" {
                    (footer())
                }
                script src=(assets::MU_JS) defer {};
                script src=(assets::SITE_JS) defer {};
            }
        }
    };

    Html(document.into_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn site_title_has_no_prefix() {
        let document = page(
            PageMetadata {
                page_title: None,
                description: "Home page",
            },
            html! {},
        )
        .0;

        assert!(document.contains("<title>Louvre</title>"));
        assert!(document.contains("<meta name=\"description\" content=\"Home page\">"));
    }

    #[test]
    fn page_title_has_site_suffix() {
        let document = page(
            PageMetadata {
                page_title: Some("Artwork"),
                description: "An artwork",
            },
            html! {},
        )
        .0;

        assert!(document.contains("<title>Artwork | Louvre</title>"));
    }
}
