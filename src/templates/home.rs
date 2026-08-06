use maud::{Markup, html};

pub fn home() -> Markup {
    html! {
        h1 { "Rust site baseline" }
        p { "Server-rendered HTML, Markdown, µJS navigation, and browser JavaScript." }
        p { a href="/articles" { "Read the sample article" } " | " a href="#animation" { "See animation" } }
        section id="animation" data-orbit-demo {
            h2 { "Anime.js" }
            svg viewBox="0 0 240 240" width="240" height="240" {
                circle cx="120" cy="120" r="90" fill="none" stroke="currentColor" {}
                circle cx="120" cy="120" r="10" fill="currentColor" {}
                rect x="112" y="20" width="16" height="16" data-orbit-node {}
            }
            p { "This runs after a normal page load and µJS navigation." }
        }
        section data-performance-metrics {
            h2 { "Browser performance" }
            dl {
                (metric("TTFB", "Time to First Byte", "Server responsiveness"))
                (metric("DOM_READY", "DOM Ready", "Document structure parsed"))
                (metric("FP", "First Paint", "Visual feedback started"))
                (metric("FCP", "First Contentful Paint", "Content begins appearing"))
                (metric("LCP", "Largest Contentful Paint", "Main content feels ready"))
                (metric("CLS", "Cumulative Layout Shift", "Visual layout is stable"))
            }
        }
    }
}

fn metric(key: &str, label: &str, description: &str) -> Markup {
    html! {
        div data-metric=(key) {
            dt { (label) }
            dd { (description) }
            dd data-metric-value { "..." }
        }
    }
}
