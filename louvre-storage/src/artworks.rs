use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, types::Json};

#[derive(Debug, Clone)]
pub struct Artwork {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub collection: Option<String>,
    pub medium: Option<String>,
    pub collection_number: Option<i32>,
    pub status: String,
    pub images: Vec<ArtworkImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkImage {
    pub alt_text: String,
    pub position: i32,
    pub source: ArtworkImageSource,
    pub web_versions: Vec<ArtworkImageVersion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkImageSource {
    pub s3_key: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkImageVersion {
    pub s3_key: String,
    pub content_type: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

#[derive(FromRow)]
struct ArtworkRow {
    slug: String,
    title: String,
    description: Option<String>,
    collection: Option<String>,
    medium: Option<String>,
    collection_number: Option<i32>,
    status: String,
    images: Json<Vec<ArtworkImage>>,
}

impl From<ArtworkRow> for Artwork {
    fn from(row: ArtworkRow) -> Self {
        Self {
            slug: row.slug,
            title: row.title,
            description: row.description,
            collection: row.collection,
            medium: row.medium,
            collection_number: row.collection_number,
            status: row.status,
            images: row.images.0,
        }
    }
}

const ARTWORK_COLUMNS: &str =
    "slug, title, description, collection, medium, collection_number, status, images";

pub async fn create_draft(
    pool: &PgPool,
    title: &str,
    slug: &str,
    description: Option<&str>,
) -> Result<Artwork, sqlx::Error> {
    let query = format!(
        "INSERT INTO artworks (title, slug, description) \
         VALUES ($1, $2, NULLIF($3, '')) RETURNING {ARTWORK_COLUMNS}"
    );
    let row = sqlx::query_as::<_, ArtworkRow>(&query)
        .bind(title)
        .bind(slug)
        .bind(description.unwrap_or_default())
        .fetch_one(pool)
        .await?;
    Ok(row.into())
}

pub async fn get_artwork(pool: &PgPool, slug: &str) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!("SELECT {ARTWORK_COLUMNS} FROM artworks WHERE slug = $1");
    sqlx::query_as::<_, ArtworkRow>(&query)
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}

pub async fn list_artworks(pool: &PgPool) -> Result<Vec<Artwork>, sqlx::Error> {
    let query =
        format!("SELECT {ARTWORK_COLUMNS} FROM artworks ORDER BY updated_at DESC, slug ASC");
    sqlx::query_as::<_, ArtworkRow>(&query)
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn list_published_artworks(pool: &PgPool) -> Result<Vec<Artwork>, sqlx::Error> {
    let query = format!(
        "SELECT {ARTWORK_COLUMNS} FROM artworks WHERE status = 'published' \
         ORDER BY collection_number ASC NULLS LAST, slug ASC"
    );
    sqlx::query_as::<_, ArtworkRow>(&query)
        .fetch_all(pool)
        .await
        .map(|rows| rows.into_iter().map(Into::into).collect())
}

pub async fn update_draft(
    pool: &PgPool,
    slug: &str,
    title: &str,
    description: Option<&str>,
) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET title = $2, description = NULLIF($3, ''), updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, ArtworkRow>(&query)
        .bind(slug)
        .bind(title)
        .bind(description.unwrap_or_default())
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}

pub async fn add_artwork_image(
    pool: &PgPool,
    slug: &str,
    alt_text: &str,
    s3_key: &str,
    content_type: &str,
) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET \
             images = images || jsonb_build_array(jsonb_build_object( \
                 'alt_text', $2, \
                 'position', jsonb_array_length(images), \
                 'source', jsonb_build_object('s3_key', $3), \
                 'web_versions', jsonb_build_array(jsonb_build_object( \
                     's3_key', $3, 'content_type', $4, 'width', NULL, 'height', NULL \
                 )) \
             )), \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, ArtworkRow>(&query)
        .bind(slug)
        .bind(alt_text)
        .bind(s3_key)
        .bind(content_type)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}

pub async fn publish_artwork(pool: &PgPool, slug: &str) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET status = 'published', \
             published_at = COALESCE(published_at, CURRENT_TIMESTAMP), \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' AND jsonb_array_length(images) > 0 \
         RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, ArtworkRow>(&query)
        .bind(slug)
        .fetch_optional(pool)
        .await
        .map(|row| row.map(Into::into))
}
