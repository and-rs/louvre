use maud::{Markup, html};

use super::{BadgeVariant, badge};

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
        name: "nvim",
        kind: "Developer tooling",
        description: "A fast Neovim configuration built around a minimal, responsive editing workflow.",
        stack: &["Zig", "Neovim", "Performance"],
        repository: "https://github.com/and-rs/nvim",
        live_url: None,
    },
    Project {
        name: "dotfiles",
        kind: "Personal platform",
        description: "A reproducible desktop environment that brings system, shell, and application configuration into one place.",
        stack: &["Nix", "QML", "Linux"],
        repository: "https://github.com/and-rs/dotfiles",
        live_url: None,
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
                h1 class="mt-2 text-4xl font-bold tracking-tight sm:text-5xl" { "Things built to be used." }
                p class="mt-4 text-lg leading-8 text-muted-foreground" { "Products, developer tools, and systems with an emphasis on speed, clarity, and staying useful over time." }
            }
            div class="mt-10 grid gap-4 md:grid-cols-2" {
                @for project in PROJECTS {
                    (project_card(project))
                }
            }
            p class="mt-8 text-sm text-muted-foreground" {
                "More experiments and open-source work live on "
                a class="font-medium text-foreground underline underline-offset-4" href="https://github.com/and-rs" target="_blank" rel="noreferrer" { "GitHub" }
                "."
            }
        }
    }
}

fn project_card(project: &Project) -> Markup {
    html! {
        article class="flex min-h-64 flex-col rounded-lg border bg-card p-6 text-card-foreground shadow-sm transition-colors group hover:bg-accent" {
            div class="flex items-start justify-between gap-4" {
                div {
                    p class="text-sm text-muted-foreground" { (project.kind) }
                    h2 class="mt-1 text-xl font-semibold tracking-tight" { (project.name) }
                }
                @if project.live_url.is_some() {
                    (badge("Live", BadgeVariant::Secondary))
                }
            }
            p class="mt-4 leading-7 text-muted-foreground" { (project.description) }
            div class="mt-5 flex flex-wrap gap-2" {
                @for item in project.stack {
                    span class="font-mono" { (badge(item, BadgeVariant::Outline)) }
                }
            }
            div class="mt-auto flex gap-4 pt-6 text-sm font-medium" {
                a class="transition-colors hover:text-muted-foreground" href=(project.repository) target="_blank" rel="noreferrer" { "Repository" }
                @if let Some(url) = project.live_url {
                    a class="transition-colors hover:text-muted-foreground" href=(url) target="_blank" rel="noreferrer" { "Visit site" }
                }
            }
        }
    }
}
