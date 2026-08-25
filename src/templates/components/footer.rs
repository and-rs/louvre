use maud::{Markup, html};

pub fn footer() -> Markup {
    html! {
        div class="border-t py-8 sm:py-10" {
            div class="flex flex-col gap-6 sm:flex-row sm:items-end sm:justify-between" {
                div class="max-w-sm" {
                    a class="text-base font-semibold tracking-tight" href="/" { "Louvre" }
                    p class="mt-2 text-sm leading-6 text-muted-foreground" {
                        "A server-rendered site baseline."
                    }
                }
                nav class="flex gap-4 text-sm font-medium" aria-label="Footer navigation" {
                    a class="text-muted-foreground transition-colors hover:text-foreground" href="/" { "Home" }
                }
            }
            div class="mt-8 border-t pt-6 text-xs text-muted-foreground" {
                p { "Copyright (c) 2026 Louvre. All Rights Reserved." }
            }
        }
    }
}
