use maud::{Markup, html};

pub fn studio(username: &str) -> Markup {
    html! {
        section class="mx-auto max-w-3xl" {
            header class="flex flex-col gap-4 border-b border-border pb-8 sm:flex-row sm:items-end sm:justify-between" {
                div {
                    p class="text-sm font-medium text-muted-foreground" { "LOUVRE / PRIVATE WORKSPACE" }
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

            section
                class="mt-8 rounded-xl border border-border bg-card p-6 sm:p-8"
                aria-labelledby="studio-welcome" {
                p class="text-sm font-medium text-muted-foreground" { "WORKSPACE" }
                h2 id="studio-welcome" class="mt-2 text-xl font-semibold tracking-tight text-card-foreground" {
                    "Welcome, " (username)
                }
                p class="mt-2 max-w-prose text-sm leading-6 text-muted-foreground" {
                    "Artwork management will live here. This private area is set up for the next step."
                }
            }
        }
    }
}

pub fn studio_login(error: Option<&str>, unavailable: bool) -> Markup {
    html! {
        section class="mx-auto max-w-md" {
            header class="mb-8" {
                p class="text-sm font-medium text-muted-foreground" { "LOUVRE / PRIVATE WORKSPACE" }
                h1 class="mt-2 page-title" { "Studio sign in" }
                p class="mt-3 text-sm leading-6 text-muted-foreground" {
                    "Sign in to manage the artwork collection."
                }
            }

            div class="rounded-xl border border-border bg-card p-6 sm:p-8" {
                @if unavailable {
                    p role="status" class="text-sm leading-6 text-muted-foreground" {
                        "Studio sign-in is not configured. Set STUDIO_USERNAME and STUDIO_PASSWORD_HASH to enable it."
                    }
                } @else {
                    @if let Some(error) = error {
                        p role="alert" class="mb-5 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" {
                            (error)
                        }
                    }

                    form method="post" action="/studio/login" class="space-y-5" {
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
                        button type="submit" class="inline-flex h-10 w-full items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground transition-colors focus-visible:outline-none hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring" {
                            "Sign in"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_form_uses_accessible_fields_and_shared_tokens() {
        let markup = studio_login(None, false).into_string();

        assert!(markup.contains("autocomplete=\"username\""));
        assert!(markup.contains("autocomplete=\"current-password\""));
        assert!(markup.contains("bg-card"));
        assert!(markup.contains("text-muted-foreground"));
    }

    #[test]
    fn studio_shows_signed_in_owner_and_logout_form() {
        let markup = studio("owner").into_string();

        assert!(markup.contains("Welcome, owner"));
        assert!(markup.contains("action=\"/studio/logout\""));
    }
}
