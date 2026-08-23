use maud::{Markup, html};

use super::icons::{desktop, star_and_crescent, sun};

pub fn theme_toggle() -> Markup {
    html! {
        button class="inline-flex size-9 items-center justify-center rounded-md transition-colors hover:bg-accent focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" type="button" aria-label="Switch theme" data-theme-toggle {
            span data-theme-icon="light" { (sun("size-6")) }
            span data-theme-icon="dark" { (star_and_crescent("size-6")) }
            span data-theme-icon="system" { (desktop("size-6")) }
        }
    }
}
