use maud::{Markup, html};

pub fn home() -> Markup {
    html! {
        div class="flex flex-col gap-12" {
            header class="max-w-2xl" {
                h1 class="page-title" { "Louvre" }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "Boilerplate home page." }
            }
        }
    }
}
