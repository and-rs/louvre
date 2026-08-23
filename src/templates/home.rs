use maud::{Markup, html};

pub fn home() -> Markup {
    html! {
        div class="mx-auto flex max-w-4xl flex-col gap-12" {
            header class="max-w-2xl" {
                h1 class="page-title" { "Sanarte" }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "Boilerplate home page." }
            }
            section {
                h2 class="text-xl font-semibold tracking-tight" { "Gallery" }
                div class="mt-4 grid grid-cols-2 gap-4 sm:grid-cols-3 lg:grid-cols-4" {
                    a href="/artwork/test" {
                        img
                            class="aspect-square w-full rounded-lg border border-border object-cover"
                            src="/artwork/test/image/1.jpg"
                            alt="Artwork test"
                            loading="lazy";
                    }
                }
            }
        }
    }
}
