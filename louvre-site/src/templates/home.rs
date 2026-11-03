use louvre_storage::Artwork;
use maud::{Markup, html};

pub fn home(artworks: &[Artwork]) -> Markup {
    html! {
        div class="mx-auto flex max-w-6xl flex-col gap-10 sm:gap-14" {
            header class="max-w-2xl border-b border-border pb-8 sm:pb-10" {
                p class="text-xs font-semibold tracking-[0.18em] text-muted-foreground uppercase" { "Selected works" }
                h1 class="mt-3 page-title" { "Louvre" }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "A collection of artworks, gathered in one place." }
            }

            @if artworks.is_empty() {
                p class="rounded-xl border border-border bg-card px-6 py-10 text-center text-muted-foreground" {
                    "The collection is being prepared. Please check back soon."
                }
            } @else {
                section class="grid grid-cols-1 gap-x-6 gap-y-10 sm:grid-cols-2 lg:grid-cols-3" aria-label="Artwork collection" {
                    @for artwork in artworks {
                        a class="block min-w-0 group" href=(format!("/artwork/{}", artwork.slug)) {
                            @if let Some(image) = artwork.images.first() {
                                @if let Some(version) = image.web_versions.first() {
                                    @let file = version.s3_key.rsplit('/').next().unwrap_or("");
                                    img
                                        class="aspect-square w-full rounded-lg border border-border bg-muted object-cover transition-opacity group-hover:opacity-90"
                                        src=(format!("/artwork/{}/image/{file}", artwork.slug))
                                        alt=(image.alt_text)
                                        loading="lazy";
                                }
                            }
                            div class="mt-4 flex items-start justify-between gap-3" {
                                h2 class="min-w-0 text-lg font-semibold tracking-tight group-hover:underline" { (artwork.title) }
                                @if let Some(collection_number) = artwork.collection_number {
                                    p class="shrink-0 text-sm text-muted-foreground" { "No. " (collection_number) }
                                }
                            }
                            @if let Some(medium) = &artwork.medium {
                                p class="mt-1 text-sm text-muted-foreground" { (medium) }
                            }
                            @if let Some(collection) = &artwork.collection {
                                p class="mt-1 text-xs text-muted-foreground" { (collection) }
                            }
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
    use louvre_storage::{ArtworkImage, ArtworkImageSource, ArtworkImageVersion};

    #[test]
    fn published_collection_links_to_artwork_and_displays_its_image() {
        let artwork = Artwork {
            slug: "blue-study".to_owned(),
            title: "Blue Study".to_owned(),
            description: None,
            collection: None,
            medium: None,
            collection_number: None,
            status: "published".to_owned(),
            images: vec![ArtworkImage {
                upload_id: None,
                alt_text: "Blue shapes on a white field".to_owned(),
                position: 0,
                source: ArtworkImageSource {
                    s3_key: "artworks/blue-study/image.png".to_owned(),
                },
                web_versions: vec![ArtworkImageVersion {
                    s3_key: "artworks/blue-study/image.png".to_owned(),
                    content_type: "image/png".to_owned(),
                    width: None,
                    height: None,
                }],
            }],
        };

        let page = home(&[artwork]).into_string();

        assert!(page.contains("href=\"/artwork/blue-study\""));
        assert!(page.contains("src=\"/artwork/blue-study/image/image.png\""));
        assert!(page.contains("alt=\"Blue shapes on a white field\""));
    }
}
