use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, LinkTarget, button_link};

pub fn not_found() -> Markup {
    html! {
        section class="max-w-lg" {
            p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { "404" }
            h1 class="mt-3 page-title" { "Page not found" }
            p class="mt-4 text-base leading-7 text-muted-foreground" { "That page does not exist." }
            p class="mt-8" { (button_link("Return home", "/", LinkTarget::CurrentTab, ButtonVariant::Primary, ButtonSize::Default, None)) }
        }
    }
}
