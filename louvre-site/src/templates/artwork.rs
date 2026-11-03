use louvre_storage::Artwork;
use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, LinkTarget, button_link};

pub fn artwork(artwork: &Artwork) -> Markup {
    html! {
        article class="mx-auto max-w-6xl" {
            p { (button_link("← Collection", "/", LinkTarget::CurrentTab, ButtonVariant::Link, ButtonSize::Small, None)) }
            div class="mt-6 grid items-start gap-8 lg:grid-cols-[minmax(0,1.3fr)_minmax(18rem,0.7fr)] lg:gap-12" {
                section data-artwork-gallery aria-label="Artwork image gallery" class="min-w-0" {
                    div data-gallery-track tabindex="0" class="relative flex aspect-square snap-x snap-mandatory overflow-x-auto scroll-smooth rounded-xl border border-border bg-muted outline-none focus-visible:ring-2 focus-visible:ring-ring" {
                        @for (index, image) in artwork.images.iter().enumerate() {
                            @if let Some(version) = image.web_versions.first() {
                                @let file = version.s3_key.rsplit('/').next().unwrap_or("");
                                figure data-gallery-slide class="h-full min-w-full snap-center" {
                                    img
                                        class="h-full w-full object-contain"
                                        src=(format!("/artwork/{}/image/{file}", artwork.slug))
                                        alt=(image.alt_text)
                                        loading=(if index == 0 { "eager" } else { "lazy" });
                                }
                            }
                        }
                        @if artwork.images.len() > 1 {
                            button data-gallery-previous type="button" aria-label="Previous image" class="absolute top-1/2 left-3 grid size-10 -translate-y-1/2 place-items-center rounded-full border border-border bg-background/90 text-xl shadow-sm hover:bg-background" { "‹" }
                            button data-gallery-next type="button" aria-label="Next image" class="absolute top-1/2 right-3 grid size-10 -translate-y-1/2 place-items-center rounded-full border border-border bg-background/90 text-xl shadow-sm hover:bg-background" { "›" }
                        }
                    }
                    @if artwork.images.len() > 1 {
                        div class="mt-3 flex items-center justify-between gap-3" {
                            p data-gallery-status role="status" aria-live="polite" class="text-sm text-muted-foreground" { "Image 1 of " (artwork.images.len()) }
                            p class="text-xs text-muted-foreground" { "Use arrow keys to browse" }
                        }
                        ol class="mt-3 grid grid-cols-4 gap-2 sm:grid-cols-6" aria-label="Choose an artwork image" {
                            @for (index, image) in artwork.images.iter().enumerate() {
                                @if let Some(version) = image.web_versions.first() {
                                    @let file = version.s3_key.rsplit('/').next().unwrap_or("");
                                    li {
                                        button data-gallery-select=(index) type="button" aria-label=(format!("Show image {}: {}", index + 1, image.alt_text)) aria-pressed=(if index == 0 { "true" } else { "false" }) class=(if index == 0 { "w-full overflow-hidden rounded-md border border-border ring-2 ring-ring" } else { "w-full overflow-hidden rounded-md border border-border" }) {
                                            img class="aspect-square w-full bg-muted object-cover" src=(format!("/artwork/{}/image/{file}", artwork.slug)) alt="" loading="lazy";
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                header class="lg:sticky lg:top-24" {
                    @if let Some(collection) = &artwork.collection {
                        p class="text-sm font-medium tracking-wide text-muted-foreground" { (collection) }
                    }
                    h1 class="mt-2 page-title" { (artwork.title) }
                    @if let Some(collection_number) = artwork.collection_number {
                        p class="mt-3 text-sm text-muted-foreground" { "No. " (collection_number) }
                    }
                    @if let Some(medium) = &artwork.medium {
                        p class="mt-5 border-t border-border pt-4 text-sm text-muted-foreground" { (medium) }
                    }
                    @if let Some(description) = &artwork.description {
                        p class="mt-5 max-w-prose leading-7 whitespace-pre-line text-muted-foreground" { (description) }
                    }
                }
            }
        }
    }
}
