use maud::{Markup, html};

pub fn home() -> Markup {
    html! {
        div class="mx-auto max-w-4xl" {
            header class="max-w-2xl" {
                h1 class="page-title" { "Sanarte" }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "Boilerplate home page." }
            }
        }
    }
}
