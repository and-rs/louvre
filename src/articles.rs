use maud::PreEscaped;
use pulldown_cmark::{Options, Parser, html::push_html};
use serde::Deserialize;
use thiserror::Error;

#[derive(Clone)]
pub struct Article {
    pub slug: &'static str,
    pub title: String,
    pub description: String,
    pub published_at: String,
    pub body: PreEscaped<String>,
}

#[derive(Deserialize)]
struct Frontmatter {
    title: String,
    description: String,
    published_at: String,
}

#[derive(Debug, Error)]
pub enum ArticleError {
    #[error("article is missing a closing frontmatter delimiter")]
    MissingFrontmatter,
    #[error("invalid article frontmatter: {0}")]
    InvalidFrontmatter(#[from] serde_yaml::Error),
}

pub fn parse_article(slug: &'static str, source: &str) -> Result<Article, ArticleError> {
    let source = source.trim();
    let frontmatter = source
        .strip_prefix("---\n")
        .ok_or(ArticleError::MissingFrontmatter)?;
    let (frontmatter, body) = frontmatter
        .split_once("\n---\n")
        .ok_or(ArticleError::MissingFrontmatter)?;
    let frontmatter: Frontmatter = serde_yaml::from_str(frontmatter)?;
    let mut html = String::new();
    push_html(&mut html, Parser::new_ext(body, Options::all()));

    Ok(Article {
        slug,
        title: frontmatter.title,
        description: frontmatter.description,
        published_at: frontmatter.published_at,
        body: PreEscaped(html),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_markdown() {
        let article = parse_article(
            "example",
            "---\ntitle: Example\ndescription: A test article.\npublished_at: 2026-08-06\n---\n\n# Hello",
        )
        .unwrap();

        assert_eq!(article.title, "Example");
        assert!(article.body.0.contains("<h1>Hello</h1>"));
    }
}
