use louvre_storage::Artwork;
use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, LinkTarget, button_link};

pub fn artwork(artwork: &Artwork) -> Markup {
    html! {
        article {
            p { (button_link("<- Collection", "/", LinkTarget::CurrentTab, ButtonVariant::Link, ButtonSize::Small, None)) }
            header class="mt-8 max-w-3xl border-b border-border pb-8" {
                h1 class="page-title" { (artwork.title) }
                @if let Some(collection) = &artwork.collection {
                    p class="mt-3 text-sm text-muted-foreground" { (collection) }
                }
                @if let Some(description) = &artwork.description {
                    p class="mt-4 max-w-prose leading-7 text-muted-foreground" { (description) }
                }
                @if let Some(medium) = &artwork.medium {
                    p class="mt-4 text-sm text-muted-foreground" { (medium) }
                }
            }
            div class="mt-8 grid gap-6 sm:grid-cols-2" {
                @for image in &artwork.images {
                    @if let Some(version) = image.web_versions.first() {
                        @let file = version.s3_key.rsplit('/').next().unwrap_or("");
                        img
                            class="w-full border border-border object-cover"
                            src=(format!("/artwork/{}/image/{file}", artwork.slug))
                            alt=(image.alt_text)
                            loading="lazy";
                    }
                }
            }
        }
    }
}
