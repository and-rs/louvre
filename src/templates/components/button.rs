use maud::{Markup, html};

#[allow(dead_code)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Ghost,
    Link,
    Destructive,
}

#[allow(dead_code)]
pub enum ButtonSize {
    Small,
    Default,
    Large,
}

pub fn button_link(label: &str, href: &str, variant: ButtonVariant, size: ButtonSize) -> Markup {
    button_link_with_target(label, href, variant, size, false)
}

#[allow(dead_code)]
pub fn external_button_link(
    label: &str,
    href: &str,
    variant: ButtonVariant,
    size: ButtonSize,
) -> Markup {
    button_link_with_target(label, href, variant, size, true)
}

fn button_link_with_target(
    label: &str,
    href: &str,
    variant: ButtonVariant,
    size: ButtonSize,
    external: bool,
) -> Markup {
    let variant_class = match variant {
        ButtonVariant::Primary => "bg-primary text-primary-foreground hover:bg-primary/90",
        ButtonVariant::Secondary => "bg-secondary text-secondary-foreground hover:bg-secondary/80",
        ButtonVariant::Ghost => "hover:bg-accent hover:text-accent-foreground",
        ButtonVariant::Link => {
            "border bg-transparent shadow-xs hover:bg-accent hover:text-accent-foreground bg-background"
        }
        ButtonVariant::Destructive => "bg-destructive text-white hover:bg-destructive/90",
    };
    let size_class = match size {
        ButtonSize::Small => "h-8 rounded-md px-4 text-xs",
        ButtonSize::Default => "h-9 rounded-md px-6 text-sm",
        ButtonSize::Large => "h-10 rounded-md px-8 text-base",
    };

    html! {
        @if external {
            a class=(format!("inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 {variant_class} {size_class}")) href=(href) target="_blank" rel="noreferrer" { (label) }
        } @else {
            a class=(format!("inline-flex items-center justify-center gap-2 whitespace-nowrap font-medium transition-colors focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-ring disabled:pointer-events-none disabled:opacity-50 {variant_class} {size_class}")) href=(href) { (label) }
        }
    }
}
