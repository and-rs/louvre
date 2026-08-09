use maud::{Markup, html};

use crate::articles::Article;

use super::{ButtonSize, ButtonVariant, button_link};

pub fn articles(article: &Article) -> Markup {
    html! {
        section class="mx-auto max-w-4xl" {
            div class="mb-8" {
                h1 class="page-title" { "Articles" }
                p class="mt-2 text-muted-foreground" { "Notes on building fast, effective websites." }
            }
            article class="rounded-lg border bg-card text-card-foreground shadow-sm transition-colors group hover:bg-accent" {
                a class="block p-6 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring focus-visible:ring-inset" href=(format!("/articles/{}", article.slug)) {
                    p class="text-sm text-muted-foreground" { (&article.published_at) }
                    h2 class="mt-2 text-xl font-semibold tracking-tight" { (&article.title) }
                    p class="mt-3 leading-7 text-muted-foreground" { (&article.description) }
                }
            }
        }
    }
}

pub fn article(article: &Article) -> Markup {
    html! {
        article class="mx-auto max-w-4xl" {
            p { (button_link("<- All articles", "/articles", ButtonVariant::Link, ButtonSize::Small)) }
            header class="mt-8 border-b pb-8" {
                p class="text-sm text-muted-foreground" { (&article.published_at) }
                h1 class="mt-2 page-title" { (&article.title) }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { (&article.description) }
            }
            div class="mt-8 leading-7 [&_a]:text-primary [&_a]:underline [&_h2]:mt-8 [&_h2]:text-2xl [&_h2]:font-semibold [&_h3]:mt-6 [&_h3]:text-xl [&_h3]:font-semibold [&_p]:mt-4" { (&article.body) }
        }
    }
}
