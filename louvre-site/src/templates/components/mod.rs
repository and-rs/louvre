#![allow(dead_code, unused_imports)]
// Components and icons are intentionally available before their first use.

mod badge;
mod button;
mod footer;
mod icons;
mod theme_toggle;

pub use badge::{BadgeVariant, badge};
pub use button::{ButtonSize, ButtonVariant, LinkTarget, button_link};
pub use footer::footer;
pub use icons::*;
pub use theme_toggle::theme_toggle;
