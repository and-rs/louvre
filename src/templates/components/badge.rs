use maud::{Markup, html};

#[allow(dead_code)]
pub enum BadgeVariant {
    Default,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

#[allow(dead_code)]
pub fn badge(label: &str, variant: BadgeVariant) -> Markup {
    let variant_class = match variant {
        BadgeVariant::Default => {
            "border-transparent bg-primary text-primary-foreground group-hover:bg-primary/90"
        }
        BadgeVariant::Secondary => {
            "border-transparent bg-secondary text-secondary-foreground group-hover:bg-secondary/80"
        }
        BadgeVariant::Destructive => {
            "border-transparent bg-destructive text-white group-hover:bg-destructive/90"
        }
        BadgeVariant::Outline => {
            "border-border text-foreground group-hover:bg-primary/80 group-hover:text-background group-hover:font-bold"
        }
        BadgeVariant::Ghost => {
            "border-transparent text-muted-foreground group-hover:bg-background group-hover:text-foreground"
        }
    };

    html! {
        span class=(format!("inline-flex items-center rounded-md border px-2 py-0.5 text-xs font-medium transition-colors {variant_class}")) { (label) }
    }
}
