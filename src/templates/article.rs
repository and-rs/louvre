use maud::{Markup, html};

use crate::articles::Article;

pub fn articles(article: &Article) -> Markup {
    html! {
        section {
            h1 { "Articles" }
            article {
                p { (&article.published_at) }
                h2 { a href=(format!("/articles/{}", article.slug)) { (&article.title) } }
                p { (&article.description) }
            }
        }
    }
}

pub fn article(article: &Article) -> Markup {
    html! {
        article {
            p { a href="/articles" { "<- All articles" } }
            p { (&article.published_at) }
            h1 { (&article.title) }
            p { (&article.description) }
            (&article.body)
        }
    }
}
