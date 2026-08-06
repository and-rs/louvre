use maud::{Markup, html};

pub fn not_found() -> Markup {
    html! {
        section class="mx-auto max-w-lg rounded-lg border bg-card p-8 text-card-foreground shadow-sm" {
            p class="text-sm font-medium text-muted-foreground" { "404" }
            h1 class="mt-2 text-3xl font-bold tracking-tight" { "Page not found" }
            p class="mt-3 text-muted-foreground" { "That page does not exist." }
            p class="mt-6" { a class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-4 py-2 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring" href="/" { "Return home" } }
        }
    }
}
