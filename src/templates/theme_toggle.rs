use maud::{Markup, html};

pub fn theme_toggle() -> Markup {
    html! {
        button class="inline-flex h-9 w-20 items-center justify-center rounded-md text-sm font-medium text-muted-foreground transition-colors hover:bg-accent hover:text-accent-foreground focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" type="button" aria-label="Switch theme" data-theme-toggle {
            span data-theme-label { "System" }
        }
    }
}
