use std::{
    env, fs,
    hash::{Hash, Hasher},
    path::Path,
};

const ASSETS: &[(&str, &str)] = &[
    (
        "APPLE_TOUCH_ICON",
        "src/static/favicon/apple-touch-icon.png",
    ),
    ("FAVICON_16", "src/static/favicon/favicon-16x16.png"),
    ("FAVICON_32", "src/static/favicon/favicon-32x32.png"),
    ("FAVICON_ICO", "src/static/favicon/favicon.ico"),
    ("FAVICON_MANIFEST", "src/static/favicon/site.webmanifest"),
    ("IRIDIUM_LOGO", "src/static/iridium.png"),
    ("ANIME_JS", "src/static/js/anime.min.js"),
    ("MU_JS", "src/static/js/mu.min.js"),
    ("SITE_CSS", "src/static/css/site.css"),
    ("SITE_JS", "src/static/js/site.js"),
];

fn main() {
    let mut output = String::new();

    for (name, path) in ASSETS {
        println!("cargo:rerun-if-changed={path}");
        let bytes = fs::read(path).expect("static asset must exist");
        let mut hasher = std::hash::DefaultHasher::new();
        bytes.hash(&mut hasher);
        let url = format!(
            "/static/{}?v={:x}",
            Path::new(path)
                .strip_prefix("src/static/")
                .unwrap()
                .display(),
            hasher.finish()
        );
        output.push_str(&format!("pub const {name}: &str = \"{url}\";\n"));
    }

    fs::write(
        Path::new(&env::var("OUT_DIR").unwrap()).join("asset_urls.rs"),
        output,
    )
    .expect("asset URL module must be written");
}
