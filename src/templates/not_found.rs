use maud::{Markup, html};

pub fn not_found() -> Markup {
    html! {
        section {
            h1 { "404" }
            p { "That page does not exist." }
            p { a href="/" { "Return home" } }
        }
    }
}
