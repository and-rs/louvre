use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, LinkTarget, button_link};

pub fn artwork(id: &str, files: &[String]) -> Markup {
    html! {
        article {
            p { (button_link("<- Gallery", "/", LinkTarget::CurrentTab, ButtonVariant::Link, ButtonSize::Small, None)) }
            header class="mt-8 border-b pb-8" {
                h1 class="page-title" { (id) }
                p class="mt-2 text-muted-foreground" { "Artwork details coming soon." }
            }
            div class="mt-8 grid gap-6 sm:grid-cols-2" {
                @for file in files {
                    img
                        class="w-full rounded-lg border border-border object-cover"
                        src=(format!("/artwork/{id}/image/{file}"))
                        alt=(file)
                        loading="lazy";
                }
            }
        }
    }
}
