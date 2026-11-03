use std::sync::LazyLock;

static SITE_NAME: LazyLock<String> =
    LazyLock::new(|| configured_name(std::env::var("SITE_NAME").ok().as_deref()).to_owned());

pub fn site_name() -> &'static str {
    SITE_NAME.as_str()
}

fn configured_name(value: Option<&str>) -> &str {
    value
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .unwrap_or("Louvre")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_or_blank_name_defaults_to_louvre() {
        for value in [None, Some(""), Some(" \n\t ")] {
            assert_eq!(configured_name(value), "Louvre");
        }
    }

    #[test]
    fn custom_name_is_trimmed_and_preserves_case() {
        assert_eq!(configured_name(Some("  Atelier & Co.  ")), "Atelier & Co.");
    }
}
