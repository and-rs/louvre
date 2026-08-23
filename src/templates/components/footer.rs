use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, arrow_up_right, external_button_link, instagram_logo};

const INSTAGRAM_URL: &str = "https://instagram.com/sanarte_galeria";

pub fn footer() -> Markup {
    html! {
        div class="border-t py-8 sm:py-10" {
            div class="grid gap-8 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end" {
                div class="max-w-sm" {
                    a class="text-base font-semibold tracking-tight" href="/" { "Sanarte" }
                    p class="mt-2 text-sm leading-6 text-muted-foreground" {
                        "Artworks and stories from the gallery."
                    }
                }
                div class="flex flex-col items-start gap-4 sm:items-end" {
                    nav class="flex gap-4 text-sm font-medium" aria-label="Footer navigation" {
                        a class="text-muted-foreground transition-colors hover:text-foreground" href="/" { "Home" }
                        a class="text-muted-foreground transition-colors hover:text-foreground" href="/#gallery" data-mu="false" { "Gallery" }
                    }
                    a class="inline-flex items-center gap-2 text-sm text-muted-foreground transition-colors hover:text-foreground" href=(INSTAGRAM_URL) target="_blank" rel="noreferrer" {
                        (instagram_logo("size-5"))
                        span { "Instagram" }
                        (arrow_up_right("size-4"))
                    }
                    (external_button_link(
                        "Contact on Instagram",
                        INSTAGRAM_URL,
                        ButtonVariant::Primary,
                        ButtonSize::Default,
                    ))
                }
            }
            div class="mt-8 border-t pt-6 text-xs text-muted-foreground" {
                p { "Copyright (c) 2026 Sanarte. All Rights Reserved." }
            }
        }
    }
}
