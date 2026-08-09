use axum::response::Html;
use maud::{DOCTYPE, Markup, PreEscaped, html};

use super::theme_toggle;

pub fn page(title: &str, description: &str, is_home: bool, content: Markup) -> Html<String> {
    let document = html! {
        (DOCTYPE)
        html lang="en" {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                meta name="description" content=(description);
                script { (PreEscaped("(function(){var theme=localStorage.getItem('theme');var dark=theme==='dark'||(!theme&&matchMedia('(prefers-color-scheme: dark)').matches);document.documentElement.classList.toggle('dark',dark);document.documentElement.style.colorScheme=dark?'dark':'light'})()")) }
                title { (title) " | Rust Site" }
                link rel="apple-touch-icon" sizes="180x180" href="/static/favicon/apple-touch-icon.png";
                link rel="icon" type="image/png" sizes="32x32" href="/static/favicon/favicon-32x32.png";
                link rel="icon" type="image/png" sizes="16x16" href="/static/favicon/favicon-16x16.png";
                link rel="icon" href="/static/favicon/favicon.ico";
                link rel="manifest" href="/static/favicon/site.webmanifest";
                link rel="preconnect" href="https://fonts.googleapis.com";
                link rel="preconnect" href="https://fonts.gstatic.com" crossorigin;
                link rel="stylesheet" href="https://fonts.googleapis.com/css2?family=Recursive:CASL,MONO,slnt,wght@0,0,0,300..1000;0,1,0,300..1000;1,0,0,300..1000;1,1,0,300..1000&display=swap";
                link rel="stylesheet" href="/static/css/site.css";
            }
            body class="min-h-screen pt-20 antialiased sm:pt-24" {
                header class="fixed inset-x-0 top-0 z-50 px-4 pt-3 sm:pt-4" {
                    nav class="mx-auto flex max-w-4xl items-center gap-1 rounded-lg border bg-background/90 p-1 shadow-sm backdrop-blur" aria-label="Main navigation" {
                        a class=(if is_home { "pointer-events-none inline-flex items-center gap-2 rounded-md px-2 py-1 opacity-0 transition-opacity hover:bg-accent" } else { "inline-flex items-center gap-2 rounded-md px-2 py-1 transition-opacity hover:bg-accent" }) href="/" data-home-link {
                            img class="size-7" src="/static/iridium.png" alt="" width="28" height="28";
                            span class="text-base font-semibold tracking-tight" { "Iridium" }
                        }
                        a class="ml-auto rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground" href="/work" { "Work" }
                        a class="rounded-md px-3 py-2 text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground" href="/articles" { "Articles" }
                        (theme_toggle())
                    }
                }
                main class="mx-auto w-full max-w-5xl px-4 py-10 sm:py-16" { (content) }
                footer class="border-t" {
                    div class="mx-auto flex max-w-5xl flex-col gap-3 px-4 py-8 text-sm text-muted-foreground sm:flex-row sm:items-center sm:justify-between" {
                        p { "Researching the future of performance." }
                        p { "Copyright (c) 2026 Iridium. All Rights Reserved." }
                        p class="flex gap-4" {
                            a class="transition-colors hover:text-foreground" href="https://www.linkedin.com/company/iridium-tech" target="_blank" rel="noreferrer" { "LinkedIn" }
                            a class="transition-colors hover:text-foreground" href="https://www.github.com/and-rs/iridium" target="_blank" rel="noreferrer" { "GitHub" }
                        }
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
