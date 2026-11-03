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
                        button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border bg-background px-4 text-sm font-medium text-foreground transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none hover:bg-accent hover:text-accent-foreground" {
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

pub struct StudioNewFields<'a> {
    pub title: &'a str,
    pub slug: &'a str,
    pub description: &'a str,
    pub collection: &'a str,
    pub medium: &'a str,
    pub collection_number: &'a str,
    pub alt_text: &'a str,
}

pub fn studio_new(error: Option<&str>, fields: StudioNewFields<'_>) -> Markup {
    html! {
        section class="max-w-2xl" {
            p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / STUDIO" }
            h1 class="mt-2 page-title" { "New artwork" }
            @if let Some(error) = error {
                p role="alert" class="mt-6 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" { (error) }
            }
            p class="mt-4 text-sm text-muted-foreground" { "Add the artwork details. You can save an incomplete draft and add or edit images before publishing." }
            form data-mu="false" method="post" enctype="multipart/form-data" action="/studio/artworks" class="mt-8 border-t border-border pt-6 space-y-6" {
                div class="grid gap-5 sm:grid-cols-2" {
                    div class="space-y-2" {
                        label for="title" class="text-sm font-medium" { "Title" }
                        input id="title" name="title" type="text" value=(fields.title) required maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                    div class="space-y-2" {
                        label for="slug" class="text-sm font-medium" { "URL slug" }
                        input id="slug" name="slug" type="text" value=(fields.slug) required pattern="[a-z0-9]+(-[a-z0-9]+)*" maxlength="100" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        p class="text-xs text-muted-foreground" { "Lowercase letters, numbers, and hyphens." }
                    }
                    div class="space-y-2" {
                        label for="collection" class="text-sm font-medium" { "Collection" }
                        input id="collection" name="collection" type="text" value=(fields.collection) maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                    div class="space-y-2" {
                        label for="medium" class="text-sm font-medium" { "Medium" }
                        input id="medium" name="medium" type="text" value=(fields.medium) maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                    div class="space-y-2" {
                        label for="collection_number" class="text-sm font-medium" { "Collection number" }
                        input id="collection_number" name="collection_number" type="number" min="1" step="1" value=(fields.collection_number) class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                }
                div class="space-y-2" {
                    label for="description" class="text-sm font-medium" { "Description" }
                    textarea id="description" name="description" rows="5" maxlength="5000" class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm" { (fields.description) }
                }
                fieldset class="border-t border-border pt-5 space-y-3" {
                    legend class="text-sm font-semibold" { "First image (optional)" }
                    p class="text-sm text-muted-foreground" { "For multiple images, create the draft and use its batch uploader." }
                    div class="space-y-2" {
                        label for="image" class="text-sm font-medium" { "Image file" }
                        input id="image" name="image" type="file" accept="image/jpeg,image/png,image/webp" class="block w-full text-sm text-muted-foreground file:mr-4 file:rounded-md file:border file:border-border file:bg-background file:px-3 file:py-2 file:text-sm file:font-medium file:text-foreground";
                        p class="text-xs text-muted-foreground" { "JPEG, PNG, or WebP, up to 10 MB." }
                    }
                    div class="space-y-2" {
                        label for="alt_text" class="text-sm font-medium" { "Alternative text" }
                        input id="alt_text" name="alt_text" type="text" value=(fields.alt_text) maxlength="500" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                    }
                }
                div class="flex gap-3" {
                    button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground hover:bg-primary/90" { "Create draft" }
                    a href="/studio" class="inline-flex h-10 items-center px-3 text-sm text-muted-foreground hover:text-foreground" { "Cancel" }
                }
            }
        }
    }
}

pub fn studio_artwork_at(artwork: &Artwork, error: Option<&str>, route_slug: &str) -> Markup {
    html! {
        section class="mx-auto max-w-5xl" {
            p class="text-xs font-semibold tracking-[0.16em] text-muted-foreground uppercase" { (site_name()) " / STUDIO" }
            div class="mt-3 flex flex-wrap items-end justify-between gap-4 border-b border-border pb-6" {
                div {
                    h1 class="page-title" { (artwork.title) }
                    p class="mt-2 text-sm text-muted-foreground" {
                        @if artwork.status == "draft" { "Draft " }
                        "/artwork/" (artwork.slug)
                    }
                }
                span class="rounded-full border border-border px-3 py-1 text-sm text-muted-foreground capitalize" { (artwork.status) }
            }
            @if let Some(error) = error {
                p role="alert" class="mt-6 rounded-md border border-destructive/30 bg-destructive/10 px-3 py-2 text-sm text-destructive" { (error) }
            }
            @if artwork.status == "draft" {
                form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}")) class="mt-8 rounded-xl border border-border bg-card p-5 sm:p-7" {
                    div class="mb-5" {
                        h2 class="text-xl font-semibold tracking-tight" { "Artwork details" }
                        p class="mt-1 text-sm text-muted-foreground" { "Add any details you have. Only the title and URL are required to save the draft." }
                    }
                    div class="grid gap-5 sm:grid-cols-2" {
                        div class="space-y-2" {
                            label for="title" class="text-sm font-medium" { "Title" }
                            input id="title" name="title" type="text" value=(artwork.title) required maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        }
                        div class="space-y-2" {
                            label for="slug" class="text-sm font-medium" { "URL slug" }
                            input id="slug" name="slug" type="text" value=(artwork.slug) required pattern="[a-z0-9]+(-[a-z0-9]+)*" maxlength="100" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                            p class="text-xs text-muted-foreground" { "Changing this updates the public URL." }
                        }
                        div class="space-y-2" {
                            label for="collection" class="text-sm font-medium" { "Collection" }
                            input id="collection" name="collection" type="text" value=(artwork.collection.as_deref().unwrap_or_default()) maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        }
                        div class="space-y-2" {
                            label for="medium" class="text-sm font-medium" { "Medium" }
                            input id="medium" name="medium" type="text" value=(artwork.medium.as_deref().unwrap_or_default()) maxlength="200" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        }
                        div class="space-y-2" {
                            label for="collection_number" class="text-sm font-medium" { "Collection number" }
                            input id="collection_number" name="collection_number" type="number" min="1" step="1" value=[artwork.collection_number] class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                        }
                        div class="sm:col-span-2 space-y-2" {
                            label for="description" class="text-sm font-medium" { "Description" }
                            textarea id="description" name="description" rows="4" maxlength="5000" class="w-full rounded-md border border-input bg-background px-3 py-2 text-sm" { (artwork.description.as_deref().unwrap_or_default()) }
                        }
                    }
                    button type="submit" class="mt-5 inline-flex h-10 items-center justify-center rounded-md border border-border bg-background px-4 text-sm font-medium hover:bg-accent" { "Save artwork details" }
                }
                section class="mt-8" aria-labelledby="upload-heading" {
                    div class="flex flex-wrap items-end justify-between gap-3" {
                        div {
                            h2 id="upload-heading" class="text-xl font-semibold tracking-tight" { "Artwork images" }
                            p class="mt-1 text-sm text-muted-foreground" {
                                (artwork.images.len()) " images · first image is the cover · each file up to 10 MB"
                                span data-saved-image-count hidden { (artwork.images.len()) }
                            }
                        }
                        @if !artwork.images.is_empty() {
                            p class="text-xs text-muted-foreground" { "Use the arrows to change image order. Alternative text is editable per image." }
                        }
                    }
                    form data-mu="false" data-image-uploader hidden method="post" enctype="multipart/form-data" action=(format!("/studio/artworks/{route_slug}/images")) class="mt-5 rounded-xl border border-dashed border-border bg-card p-5 sm:p-7" {
                        label for="batch-images" class="flex min-h-32 cursor-pointer flex-col items-center justify-center rounded-lg bg-muted/50 p-6 text-center focus-within:ring-2 focus-within:ring-ring" {
                            span class="font-medium" { "Choose images or drop them here" }
                            span class="mt-1 text-sm text-muted-foreground" { "JPEG, PNG, WebP · up to 10 MB per image" }
                            input id="batch-images" data-batch-files name="image" type="file" accept="image/jpeg,image/png,image/webp" multiple class="mt-4 block w-full max-w-xs text-sm text-muted-foreground file:mr-4 file:rounded-md file:border file:border-border file:bg-background file:px-3 file:py-2 file:text-sm file:font-medium file:text-foreground";
                        }
                        div data-dropzone class="mt-4 hidden rounded-md border border-border p-3 text-center text-sm text-muted-foreground" { "Drop image files to add them to this batch" }
                        ul data-upload-queue class="mt-5 grid gap-3 sm:grid-cols-2" aria-label="Selected images" {}
                        p data-upload-status role="status" aria-live="polite" class="mt-3 text-sm text-muted-foreground" {}
                        div class="mt-4 flex flex-wrap gap-3" {
                            button data-upload-submit type="submit" disabled class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50" { "Upload selected images" }
                        }
                    }
                    noscript {
                        form data-mu="false" method="post" enctype="multipart/form-data" action=(format!("/studio/artworks/{route_slug}/images")) class="mt-5 rounded-xl border border-border p-5 space-y-4" {
                            p class="text-sm text-muted-foreground" { "Upload one image at a time with JavaScript disabled." }
                            div class="space-y-2" {
                                label for="image" class="text-sm font-medium" { "Image file" }
                                input id="image" name="image" type="file" accept="image/jpeg,image/png,image/webp" required class="block w-full text-sm text-muted-foreground";
                            }
                            div class="space-y-2" {
                                label for="alt_text" class="text-sm font-medium" { "Alternative text" }
                                input id="alt_text" name="alt_text" type="text" required maxlength="500" class="h-10 w-full rounded-md border border-input bg-background px-3 text-sm";
                            }
                            button type="submit" class="inline-flex h-9 items-center justify-center rounded-md border border-border px-4 text-sm font-medium hover:bg-accent" { "Upload image" }
                        }
                    }
                    @if artwork.images.is_empty() {
                        p class="mt-5 rounded-lg border border-border bg-muted/40 p-6 text-sm text-muted-foreground" { "No images yet. Add at least one image before publishing." }
                    } @else {
                        ol class="mt-5 grid gap-5 sm:grid-cols-2 lg:grid-cols-3" aria-label="Artwork images" {
                            @for (index, image) in artwork.images.iter().enumerate() {
                                @let version = image.web_versions.first();
                                li data-image-position=(index) class="overflow-hidden rounded-xl border border-border bg-card" {
                                    @if let Some(version) = version {
                                        img class="aspect-square w-full bg-muted object-cover" src=(format!("/studio/artworks/{route_slug}/images/{index}/thumbnail")) alt=(image.alt_text) loading="lazy";
                                        div class="p-4 space-y-4" {
                                            div class="flex items-center justify-between gap-2" {
                                                p class="text-sm font-semibold" { @if index == 0 { "Cover image" } @else { "Image " (index + 1) } }
                                                p class="text-xs text-muted-foreground" {
                                                    @if let (Some(width), Some(height)) = (version.width, version.height) {
                                                        (width) " × " (height)
                                                    } @else {
                                                        "Dimensions unavailable"
                                                    }
                                                }
                                            }
                                            form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}/images/{index}/alt-text")) class="space-y-2" {
                                                label for=(format!("alt-text-{index}")) class="text-xs font-medium text-muted-foreground" { "Alternative text" }
                                                input id=(format!("alt-text-{index}")) name="alt_text" type="text" value=(image.alt_text) required maxlength="500" class="h-9 w-full rounded-md border border-input bg-background px-3 text-sm";
                                                button type="submit" class="text-sm font-medium underline underline-offset-4 hover:text-muted-foreground" { "Save alt text" }
                                            }
                                            div class="flex flex-wrap items-center justify-between gap-2 border-t border-border pt-3" {
                                                div class="flex gap-2" {
                                                    form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}/images/{index}/move")) {
                                                        input type="hidden" name="direction" value="up";
                                                        button type="submit" disabled[index == 0] aria-label=(format!("Move image {} earlier", index + 1)) class="size-9 rounded-md border border-border text-lg hover:bg-accent disabled:opacity-40" { "↑" }
                                                    }
                                                    form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}/images/{index}/move")) {
                                                        input type="hidden" name="direction" value="down";
                                                        button type="submit" disabled[index + 1 == artwork.images.len()] aria-label=(format!("Move image {} later", index + 1)) class="size-9 rounded-md border border-border text-lg hover:bg-accent disabled:opacity-40" { "↓" }
                                                    }
                                                }
                                                form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}/images/{index}/delete")) onsubmit="return confirm('Remove this image?')" {
                                                    button type="submit" class="rounded-md px-2 py-2 text-sm text-destructive hover:bg-destructive/10" { "Remove" }
                                                }
                                            }
                                        }
                                    } @else {
                                        div class="aspect-square bg-muted" {}
                                        p class="p-4 text-sm text-muted-foreground" { "Image preview unavailable" }
                                    }
                                }
                            }
                        }
                    }
                }
                form data-mu="false" method="post" action=(format!("/studio/artworks/{route_slug}/publish")) class="mt-8 border-t border-border pt-6" {
                    button data-publish-button type="submit" disabled[artwork.images.is_empty()] class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground hover:bg-primary/90 disabled:pointer-events-none disabled:opacity-50" { "Publish artwork" }
                    @if artwork.images.is_empty() {
                        p class="mt-2 text-sm text-muted-foreground" { "An image is required to publish. Artwork details remain optional." }
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
                    input id="username" name="username" type="text" autocomplete="username" required autofocus class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground";
                }
                div class="space-y-2" {
                    label for="password" class="text-sm font-medium text-foreground" { "Password" }
                    input id="password" name="password" type="password" autocomplete="current-password" required class="flex h-10 w-full rounded-md border border-input bg-background px-3 py-2 text-sm text-foreground transition-shadow outline-none focus-visible:ring-2 focus-visible:ring-ring placeholder:text-muted-foreground";
                }
                button type="submit" class="inline-flex h-10 items-center justify-center rounded-md bg-primary px-5 text-sm font-medium text-primary-foreground transition-colors focus-visible:ring-2 focus-visible:ring-ring focus-visible:outline-none hover:bg-primary/90" {
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
        let page = studio_artwork_at(&draft(), None, "blue-study").into_string();

        assert!(page.contains("enctype=\"multipart/form-data\""));
        assert!(page.contains("/studio/artworks/blue-study/images"));
        assert!(page.contains("/studio/artworks/blue-study/publish"));
        assert!(page.contains("An image is required to publish."));
    }

    #[test]
    fn published_page_links_to_public_artwork_without_upload_controls() {
        let mut artwork = draft();
        artwork.status = "published".to_owned();
        let page = studio_artwork_at(&artwork, None, "blue-study").into_string();

        assert!(page.contains("href=\"/artwork/blue-study\""));
        assert!(!page.contains("multipart/form-data"));
        assert!(!page.contains("/publish\""));
    }
}
