use maud::{Markup, html};

use super::{ButtonSize, ButtonVariant, LinkTarget, arrow_up_right, button_link, github_logo};

const LOUVRE_GITHUB_URL: &str = "https://github.com/and-rs/louvre";
const AND_RS_GITHUB_URL: &str = "https://github.com/and-rs";

pub fn footer() -> Markup {
    html! {
        div class="rounded-lg border p-2 shadow-sm backdrop-blur" {
            div class="flex flex-col gap-6 sm:flex-row sm:items-end sm:justify-between" {
                div class="max-w-sm px-2" {
                    a class="text-base font-semibold tracking-tight" href="/" { "Louvre" }
                    span class="text-sm leading-6 text-muted-foreground" {
                        p {
                            "A server-rendered site baseline."
                        }
                        p {
                            "Copyright (c) 2026 Louvre. All Rights Reserved."
                        }
                    }
                }
                div class="flex flex-col items-start gap-4 sm:items-end" {
                    nav class="flex gap-4 text-sm font-medium" aria-label="Footer navigation" {
                        (button_link(
                                "Home",
                                "/",
                                LinkTarget::CurrentTab,
                                ButtonVariant::Link,
                                ButtonSize::Default,
                                None,
                        ))
                    }
                    (button_link(
                        "More OSS by me",
                        AND_RS_GITHUB_URL,
                        LinkTarget::NewTab,
                        ButtonVariant::Link,
                        ButtonSize::Default,
                        None,
                    ))
                    (button_link(
                        html! {
                            (github_logo("size-5"))
                            span { "Louvre on GitHub" }
                            (arrow_up_right("size-4"))
                        },
                        LOUVRE_GITHUB_URL,
                        LinkTarget::NewTab,
                        ButtonVariant::Link,
                        ButtonSize::Default,
                        None,
                    ))
                }
            }
        }
    }
}
