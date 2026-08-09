use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, button_link};

pub fn not_found() -> Markup {
    html! {
        section class="mx-auto max-w-lg rounded-lg border bg-card p-8 text-card-foreground shadow-sm" {
            p class="text-sm font-medium text-muted-foreground" { "404" }
            h1 class="mt-2 text-3xl font-bold tracking-tight" { "Page not found" }
            p class="mt-3 text-muted-foreground" { "That page does not exist." }
            p class="mt-6" { (button_link("Return home", "/", ButtonVariant::Primary, ButtonSize::Default)) }
        }
    }
}
