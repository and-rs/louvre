use maud::{Markup, html};

pub fn studio(username: &str) -> Markup {
    html! {
        section {
            header class="flex flex-col gap-4 border-b border-border pb-8 sm:flex-row sm:items-end sm:justify-between" {
                div {
                    p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { "LOUVRE / PRIVATE WORKSPACE" }
                    h1 class="mt-2 page-title" { "Studio" }
                    p class="mt-3 max-w-xl text-muted-foreground" {
                        "A quiet workspace for preparing and publishing your artwork."
                    }
                }
                form method="post" action="/studio/logout" {
                    button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border bg-background px-4 text-sm font-medium text-foreground transition-colors focus-visible:outline-none hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring" {
                        "Sign out"
                    }
                }
            }

            section class="mt-10 max-w-xl" aria-labelledby="studio-welcome" {
                p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { "WORKSPACE" }
                h2 id="studio-welcome" class="mt-3 text-2xl font-semibold tracking-tight" {
                    "Welcome, " (username)
                }
                p class="mt-3 max-w-prose leading-7 text-muted-foreground" {
                    "Artwork management will live here. This private area is set up for the next step."
                }
            }
        }
    }
}

pub fn studio_login(error: Option<&str>) -> Markup {
    html! {
        section class="max-w-2xl" {
            header {
                p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { "LOUVRE / PRIVATE WORKSPACE" }
                h1 class="mt-2 page-title" { "Studio sign in" }
                p class="mt-4 max-w-xl text-base leading-7 text-muted-foreground" {
                    "Sign in to manage the artwork collection."
                }
            }

            @if let Some(error) = error {
                p role="alert" class="mt-8 max-w-md rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" {
                    (error)
                }
            }

            form method="post" action="/studio/login" class="mt-10 max-w-md border-t border-border pt-6 space-y-6" {
                div class="space-y-2" {
                    label for="username" class="text-sm font-medium text-foreground" { "Username" }
                    input
                        id="username"
                        name="username"
                        type="text"
                        autocomplete="username"
                        required
                        autofocus
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring";
                }
                div class="space-y-2" {
                    label for="password" class="text-sm font-medium text-foreground" { "Password" }
                    input
                        id="password"
                        name="password"
                        type="password"
                        autocomplete="current-password"
                        required
                        class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring";
                }
                button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground transition-colors focus-visible:outline-none hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring" {
                    "Sign in"
                }
            }
        }
    }
}
