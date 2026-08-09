use maud::{Markup, html};

use super::{BadgeVariant, ButtonSize, ButtonVariant, badge, external_button_link};

struct Project {
    name: &'static str,
    kind: &'static str,
    description: &'static str,
    stack: &'static [&'static str],
    repository: &'static str,
    live_url: Option<&'static str>,
}

const PROJECTS: &[Project] = &[
    Project {
        name: "edge",
        kind: "Market intelligence",
        description: "Event-to-market intelligence for crypto and prediction-market operators.",
        stack: &["TypeScript", "Markets", "Product"],
        repository: "https://github.com/and-rs/edge",
        live_url: Some("https://edge.baut.dev"),
    },
    Project {
        name: "wisp",
        kind: "AI tooling",
        description: "A terminal-native AI harness focused on a direct, programmable command-line workflow.",
        stack: &["Zig", "Terminal", "AI"],
        repository: "https://github.com/and-rs/wisp",
        live_url: None,
    },
    Project {
        name: "instagram-framer",
        kind: "Creative automation",
        description: "A workflow that frames artwork, writes a caption, and publishes the finished post to Instagram.",
        stack: &["Python", "Automation", "Media"],
        repository: "https://github.com/and-rs/instagram-framer",
        live_url: None,
    },
];

pub fn work() -> Markup {
    html! {
        section class="mx-auto max-w-4xl" {
            header class="max-w-2xl" {
                p class="text-sm font-medium text-muted-foreground" { "Selected work" }
                h1 class="mt-2 page-title" { "Things built to be used." }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "Products, developer tools, and systems with an emphasis on speed, clarity, and staying useful over time." }
            }
            div class="mt-10 flex flex-col gap-4" {
                @for project in PROJECTS {
                    (project_card(project))
                }
            }
            p class="mt-8 text-sm text-muted-foreground" {
                "More experiments and open-source work live on "
                (external_button_link("GitHub", "https://github.com/and-rs", ButtonVariant::Link, ButtonSize::Small))
                "."
            }
        }
    }
}

fn project_card(project: &Project) -> Markup {
    html! {
        article class="flex min-h-60 flex-col gap-4 rounded-lg border bg-card p-6 text-card-foreground shadow-sm transition-colors group" {
            div class="flex items-start justify-between gap-4" {
                div {
                    p class="text-sm text-muted-foreground" { (project.kind) }
                    h2 class="mt-1 text-xl font-semibold tracking-tight" { (project.name) }
                }
                @if project.live_url.is_some() {
                    (badge("Live", BadgeVariant::Secondary))
                }
            }
            div class="flex flex-wrap gap-2" {
                @for item in project.stack {
                    span class="font-mono" { (badge(item, BadgeVariant::Outline)) }
                }
            }
            p class="leading-7 text-muted-foreground" { (project.description) }
            div class="flex gap-4 text-sm font-medium" {
                (external_button_link("Repository", project.repository, ButtonVariant::Link, ButtonSize::Small))
                @if let Some(url) = project.live_url {
                    (external_button_link("Visit site", url, ButtonVariant::Link, ButtonSize::Small))
                }
            }
        }
    }
}
