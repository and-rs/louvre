use louvre_tw_merge::merge_classes;
use maud::{Markup, Render, html};

pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Link,
    Destructive,
}

pub enum ButtonSize {
    Small,
    Default,
    Large,
}

pub enum LinkTarget {
    CurrentTab,
    NewTab,
}

pub fn button_link(
    content: impl Render,
    href: &str,
    target: LinkTarget,
    variant: ButtonVariant,
    size: ButtonSize,
    class: Option<&str>,
) -> Markup {
    let variant_class = match variant {
        ButtonVariant::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Link => {
            "bg-transparent hover:bg-accent/40 hover:text-accent-foreground bg-background hover:underline underline-offset-3"
        }
        ButtonVariant::Destructive => "bg-destructive text-white hover:bg-destructive/90",
    };
    let size_class = match size {
        ButtonSize::Small => "h-8 rounded-md px-2 text-xs",
        ButtonSize::Default => "h-9 rounded-md px-4 text-sm",
        ButtonSize::Large => "h-10 rounded-md px-6 text-base",
    };
    let class = merge_classes(&[
        "inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50",
        variant_class,
        size_class,
        class.unwrap_or_default(),
    ]);

    html! {
        @match target {
            LinkTarget::CurrentTab => {
                a class=(class) href=(href) { (content) }
            }
            LinkTarget::NewTab => {
                a class=(class) href=(href) target="_blank" rel="noreferrer" { (content) }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use maud::html;

    #[test]
    fn class_override_replaces_conflicting_utilities() {
        let button = button_link(
            "Gallery",
            "/",
            LinkTarget::CurrentTab,
            ButtonVariant::Primary,
            ButtonSize::Default,
            Some("text-lg px-2 hover:bg-accent"),
        )
        .into_string();

        assert!(button.contains("text-lg"));
        assert!(button.contains("px-2"));
        assert!(button.contains("hover:bg-accent"));
        assert!(!button.contains("text-sm"));
        assert!(!button.contains("px-6"));
        assert!(!button.contains("hover:bg-primary/90"));
    }

    #[test]
    fn new_tab_link_renders_markup_content() {
        let button = button_link(
            html! {
                span { "Repository" }
                span aria-hidden="true" { "->" }
            },
            "https://github.com/and-rs/louvre",
            LinkTarget::NewTab,
            ButtonVariant::Ghost,
            ButtonSize::Default,
            None,
        )
        .into_string();

        assert!(button.contains("<span>Repository</span>"));
        assert!(button.contains("target=\"_blank\""));
        assert!(button.contains("rel=\"noreferrer\""));
    }
}
