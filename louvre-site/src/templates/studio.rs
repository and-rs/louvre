use louvre_storage::Artwork;
use maud::{Markup, html};

use crate::branding::site_name;

pub fn studio(username: &str, artworks: &[Artwork]) -> Markup {
    html! {
        section {
            header class="flex flex-col gap-4 border-b border-border pb-8 sm:flex-row sm:items-end sm:justify-between" {
                div {
                    p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / PRIVATE WORKSPACE" }
                    h1 class="mt-2 page-title" { "Studio" }
                    p class="mt-3 max-w-xl text-muted-foreground" { "Welcome, " (username) ". Manage your artwork drafts and published collection." }
                }
                div class="flex gap-2" {
                    a href="/studio/artworks/new" class="inline-flex h-9 items-center justify-center rounded-md bg-primary px-4 text-sm font-medium text-primary-foreground transition-colors hover:bg-primary/90" { "New artwork" }
                    form method="post" action="/studio/logout" {
                        button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border bg-background px-4 text-sm font-medium text-foreground transition-colors focus-visible:outline-none hover:bg-accent hover:text-accent-foreground focus-visible:ring-2 focus-visible:ring-ring" {
                            "Sign out"
                        }
                    }
                }
            }
            section class="mt-10" aria-labelledby="artwork-list-heading" {
                div class="flex items-end justify-between border-b border-border pb-4" {
                    h2 id="artwork-list-heading" class="text-xl font-semibold tracking-tight" { "Artwork" }
                    p class="text-sm text-muted-foreground" { (artworks.len()) " total" }
                }
                @if artworks.is_empty() {
                    p class="py-8 text-muted-foreground" { "No artwork yet. Create a draft to get started." }
                } @else {
                    ul class="divide-y divide-border" {
                        @for artwork in artworks {
                            li class="flex items-center gap-4 py-5" {
                                @if artwork.images.first().and_then(|image| image.web_versions.first()).is_some() {
                                    img class="size-16 shrink-0 rounded-md border border-border object-cover" src=(format!("/studio/artworks/{}/thumbnail", artwork.slug)) alt="" aria-hidden="true" loading="lazy";
                                } @else {
                                    div role="img" aria-label="No image uploaded" class="size-16 shrink-0 rounded-md border border-border bg-muted" {}
                                }
                                div class="min-w-0 flex-1" {
                                    a class="font-medium hover:underline" href=(format!("/studio/artworks/{}", artwork.slug)) { (artwork.title) }
                                    p class="mt-1 text-sm text-muted-foreground" { "/artwork/" (artwork.slug) }
                                }
                                p class="shrink-0 text-sm text-muted-foreground capitalize" { (artwork.status) " · " (artwork.images.len()) " images" }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn studio_new(
    error: Option<&str>,
    title: &str,
    slug: &str,
    description: &str,
    alt_text: &str,
) -> Markup {
    html! {
        section class="max-w-2xl" {
            p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / STUDIO" }
            h1 class="mt-2 page-title" { "New artwork" }
            @if let Some(error) = error {
                p role="alert" class="mt-6 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" { (error) }
            }
            p class="mt-4 text-sm text-muted-foreground" { "Add the first image now or create the draft and upload one later. At least one image is required to publish." }
            form method="post" enctype="multipart/form-data" action="/studio/artworks" class="mt-8 border-t border-border pt-6 space-y-6" {
                div class="space-y-2" {
                    label for="title" class="text-sm font-medium" { "Title" }
                    input id="title" name="title" type="text" value=(title) required maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                }
                div class="space-y-2" {
                    label for="slug" class="text-sm font-medium" { "URL slug" }
                    input id="slug" name="slug" type="text" value=(slug) required pattern="[a-z0-9]+(-[a-z0-9]+)*" maxlength="100" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    p class="text-xs text-muted-foreground" { "Lowercase letters, numbers, and single hyphens." }
                }
                div class="space-y-2" {
                    label for="description" class="text-sm font-medium" { "Description (optional)" }
                    textarea id="description" name="description" rows="5" maxlength="5000" class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm" { (description) }
                }
                div class="space-y-2" {
                    label for="image" class="text-sm font-medium" { "First image (optional)" }
                    input id="image" name="image" type="file" accept="image/jpeg,image/png,image/webp" class="block w-full text-sm text-muted-foreground file:mr-4 file:rounded-md file:border file:border-border file:bg-background file:px-3 file:py-2 file:text-sm file:font-medium file:text-foreground";
                    p class="text-xs text-muted-foreground" { "JPEG, PNG, or WebP, up to 10 MB." }
                }
                div class="space-y-2" {
                    label for="alt_text" class="text-sm font-medium" { "Alternative text (required with an image)" }
                    input id="alt_text" name="alt_text" type="text" value=(alt_text) maxlength="500" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                }
                div class="flex gap-3" {
                    button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground hover:bg-primary/90" { "Create draft" }
                    a href="/studio" class="inline-flex h-10 items-center px-3 text-sm text-muted-foreground hover:text-foreground" { "Cancel" }
                }
            }
        }
    }
}

pub fn studio_artwork(artwork: &Artwork, error: Option<&str>) -> Markup {
    html! {
        section class="max-w-3xl" {
            p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / STUDIO" }
            div class="mt-2 flex flex-wrap items-baseline justify-between gap-3" {
                h1 class="page-title" { (artwork.title) }
                p class="text-sm text-muted-foreground capitalize" { (artwork.status) }
            }
            p class="mt-2 text-sm text-muted-foreground" { "/artwork/" (artwork.slug) }
            @if let Some(error) = error {
                p role="alert" class="mt-6 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" { (error) }
            }
            @if artwork.status == "draft" {
                form method="post" action=(format!("/studio/artworks/{}", artwork.slug)) class="mt-8 border-t border-border pt-6 space-y-5" {
                    div class="space-y-2" {
                        label for="title" class="text-sm font-medium" { "Title" }
                        input id="title" name="title" type="text" value=(artwork.title) required maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                    div class="space-y-2" {
                        label for="description" class="text-sm font-medium" { "Description (optional)" }
                        textarea id="description" name="description" rows="4" maxlength="5000" class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm" { (artwork.description.as_deref().unwrap_or_default()) }
                    }
                    button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border px-4 text-sm font-medium hover:bg-accent" { "Save details" }
                }
                section class="mt-10 border-t border-border pt-6" aria-labelledby="upload-heading" {
                    h2 id="upload-heading" class="text-xl font-semibold tracking-tight" { "Images" }
                    p class="mt-2 text-sm text-muted-foreground" { "Upload JPEG, PNG, or WebP images (up to 10 MB each). Images are stored as uploaded." }
                    form method="post" enctype="multipart/form-data" action=(format!("/studio/artworks/{}/images", artwork.slug)) class="mt-5 space-y-4" {
                        div class="space-y-2" {
                            label for="image" class="text-sm font-medium" { "Image file" }
                            input id="image" name="image" type="file" accept="image/jpeg,image/png,image/webp" required class="block w-full text-sm text-muted-foreground file:mr-4 file:rounded-md file:border file:border-border file:bg-background file:px-3 file:py-2 file:text-sm file:font-medium file:text-foreground";
                        }
                        div class="space-y-2" {
                            label for="alt_text" class="text-sm font-medium" { "Alternative text" }
                            input id="alt_text" name="alt_text" type="text" required maxlength="500" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        }
                        button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border px-4 text-sm font-medium hover:bg-accent" { "Upload image" }
                    }
                    @if artwork.images.is_empty() {
                        p class="mt-6 text-sm text-muted-foreground" { "No images uploaded yet." }
                    } @else {
                        ol class="mt-6 border-y border-border divide-y divide-border" {
                            @for image in &artwork.images {
                                li class="py-3" {
                                    p class="text-sm font-medium" { (image.alt_text) }
                                    p class="mt-1 text-xs break-all text-muted-foreground" { (image.source.s3_key) }
                                }
                            }
                        }
                    }
                }
                form method="post" action=(format!("/studio/artworks/{}/publish", artwork.slug)) class="mt-8 border-t border-border pt-6" {
                    button type="submit" disabled[artwork.images.is_empty()] class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50" { "Publish artwork" }
                    @if artwork.images.is_empty() {
                        p class="mt-2 text-sm text-muted-foreground" { "Upload at least one image before publishing." }
                    }
                }
            } @else if artwork.status == "published" {
                p class="mt-8 border-t border-border pt-6 text-sm text-muted-foreground" {
                    "This artwork is public. "
                    a class="underline underline-offset-4 hover:text-foreground" href=(format!("/artwork/{}", artwork.slug)) { "View it on the site." }
                }
            }
            p class="mt-8" {
                a class="text-sm text-muted-foreground underline underline-offset-4 hover:text-foreground" href="/studio" { "Back to Studio" }
            }
        }
    }
}

pub fn studio_login(error: Option<&str>) -> Markup {
    html! {
        section class="max-w-2xl" {
            header {
                p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / PRIVATE WORKSPACE" }
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
                    input id="username" name="username" type="text" autocomplete="username" required autofocus class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring";
                }
                div class="space-y-2" {
                    label for="password" class="text-sm font-medium text-foreground" { "Password" }
                    input id="password" name="password" type="password" autocomplete="current-password" required class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none placeholder:text-muted-foreground focus-visible:ring-2 focus-visible:ring-ring";
                }
                button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground transition-colors focus-visible:outline-none hover:bg-primary/90 focus-visible:ring-2 focus-visible:ring-ring" {
                    "Sign in"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn draft() -> Artwork {
        Artwork {
            slug: "blue-study".to_owned(),
            title: "Blue Study".to_owned(),
            description: None,
            collection: None,
            medium: None,
            collection_number: None,
            status: "draft".to_owned(),
            images: Vec::new(),
        }
    }

    #[test]
    fn draft_page_has_upload_and_publish_controls() {
        let page = studio_artwork(&draft(), None).into_string();

        assert!(page.contains("enctype=\"multipart/form-data\""));
        assert!(page.contains("/studio/artworks/blue-study/images"));
        assert!(page.contains("/studio/artworks/blue-study/publish"));
        assert!(page.contains("Upload at least one image before publishing."));
    }

    #[test]
    fn published_page_links_to_public_artwork_without_upload_controls() {
        let mut artwork = draft();
        artwork.status = "published".to_owned();
        let page = studio_artwork(&artwork, None).into_string();

        assert!(page.contains("href=\"/artwork/blue-study\""));
        assert!(!page.contains("multipart/form-data"));
        assert!(!page.contains("/publish\""));
    }
}
