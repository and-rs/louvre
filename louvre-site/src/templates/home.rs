use louvre_storage::Artwork;
use maud::{Markup, html};

pub fn home(artworks: &[Artwork]) -> Markup {
    html! {
        div class="flex flex-col gap-12" {
            header class="max-w-2xl" {
                h1 class="page-title" { "Louvre" }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "An artwork publishing platform." }
            }

            @if artworks.is_empty() {
                p class="border-t border-border pt-6 text-muted-foreground" {
                    "The collection is being prepared. Please check back soon."
                }
            } @else {
                section class="grid gap-8 sm:grid-cols-2" aria-label="Artwork collection" {
                    @for artwork in artworks {
                        a class="block group" href=(format!("/artwork/{}", artwork.slug)) {
                            @if let Some(image) = artwork.images.first() {
                                @if let Some(version) = image.web_versions.first() {
                                    @let file = version.s3_key.rsplit('/').next().unwrap_or("");
                                    img
                                        class="aspect-[4/3] w-full border border-border object-cover transition-opacity group-hover:opacity-90"
                                        src=(format!("/artwork/{}/image/{file}", artwork.slug))
                                        alt=(image.alt_text)
                                        loading="lazy";
                                }
                            }
                            h2 class="mt-4 text-lg font-medium tracking-tight group-hover:underline" { (artwork.title) }
                            @if let Some(collection_number) = artwork.collection_number {
                                p class="mt-1 text-sm text-muted-foreground" { "No. " (collection_number) }
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
