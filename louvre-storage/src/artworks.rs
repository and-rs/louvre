use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, types::Json};

#[derive(Debug, Clone, FromRow)]
pub struct Artwork {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub collection: Option<String>,
    pub medium: Option<String>,
    pub collection_number: Option<i32>,
    pub status: String,
    #[sqlx(json)]
    pub images: Vec<ArtworkImage>,
}

#[derive(Debug, Clone)]
pub struct ArtworkDetails {
    pub title: String,
    pub slug: String,
    pub description: Option<String>,
    pub collection: Option<String>,
    pub medium: Option<String>,
    pub collection_number: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkImage {
    #[serde(default)]
    pub upload_id: Option<String>,
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

pub struct ArtworkImageInput<'a> {
    pub alt_text: &'a str,
    pub s3_key: &'a str,
    pub content_type: &'a str,
    pub upload_id: &'a str,
    pub width: u32,
    pub height: u32,
}

const ARTWORK_COLUMNS: &str =
    "slug, title, description, collection, medium, collection_number, status, images";

pub async fn create_draft(pool: &PgPool, details: &ArtworkDetails) -> Result<Artwork, sqlx::Error> {
    let query = format!(
        "INSERT INTO artworks (title, slug, description, collection, medium, collection_number) \
         VALUES ($1, $2, NULLIF($3, ''), NULLIF($4, ''), NULLIF($5, ''), $6) \
         RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, Artwork>(&query)
        .bind(&details.title)
        .bind(&details.slug)
        .bind(details.description.as_deref().unwrap_or_default())
        .bind(details.collection.as_deref().unwrap_or_default())
        .bind(details.medium.as_deref().unwrap_or_default())
        .bind(details.collection_number)
        .fetch_one(pool)
        .await
}

pub async fn get_artwork(pool: &PgPool, slug: &str) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!("SELECT {ARTWORK_COLUMNS} FROM artworks WHERE slug = $1");
    sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .fetch_optional(pool)
        .await
}

pub async fn list_artworks(pool: &PgPool) -> Result<Vec<Artwork>, sqlx::Error> {
    let query =
        format!("SELECT {ARTWORK_COLUMNS} FROM artworks ORDER BY updated_at DESC, slug ASC");
    sqlx::query_as::<_, Artwork>(&query).fetch_all(pool).await
}

pub async fn list_published_artworks(pool: &PgPool) -> Result<Vec<Artwork>, sqlx::Error> {
    let query = format!(
        "SELECT {ARTWORK_COLUMNS} FROM artworks WHERE status = 'published' \
         ORDER BY collection_number ASC NULLS LAST, slug ASC"
    );
    sqlx::query_as::<_, Artwork>(&query).fetch_all(pool).await
}

pub async fn update_draft(
    pool: &PgPool,
    slug: &str,
    details: &ArtworkDetails,
) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET slug = $2, title = $3, description = NULLIF($4, ''), \
             collection = NULLIF($5, ''), medium = NULLIF($6, ''), collection_number = $7, \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .bind(&details.slug)
        .bind(&details.title)
        .bind(details.description.as_deref().unwrap_or_default())
        .bind(details.collection.as_deref().unwrap_or_default())
        .bind(details.medium.as_deref().unwrap_or_default())
        .bind(details.collection_number)
        .fetch_optional(pool)
        .await
}

pub async fn add_artwork_image(
    pool: &PgPool,
    slug: &str,
    image: ArtworkImageInput<'_>,
) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET \
             images = images || jsonb_build_array(jsonb_build_object( \
                  'alt_text', $2, \
                  'position', jsonb_array_length(images), \
                  'upload_id', $5, \
                  'source', jsonb_build_object('s3_key', $3), \
                  'web_versions', jsonb_build_array(jsonb_build_object( \
                     's3_key', $3, 'content_type', $4, 'width', $6, 'height', $7 \
                 )) \
             )), \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' \
             AND NOT images @> jsonb_build_array(jsonb_build_object('upload_id', $5)) \
         RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .bind(image.alt_text)
        .bind(image.s3_key)
        .bind(image.content_type)
        .bind(image.upload_id)
        .bind(image.width as i64)
        .bind(image.height as i64)
        .fetch_optional(pool)
        .await
}

pub async fn update_artwork_image_alt_text(
    pool: &PgPool,
    slug: &str,
    position: usize,
    alt_text: &str,
) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET images = jsonb_set(images, ARRAY[$2::text, 'alt_text'], to_jsonb($3::text)), \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' AND $2 < jsonb_array_length(images) \
         RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .bind(position as i64)
        .bind(alt_text)
        .fetch_optional(pool)
        .await
}

pub async fn move_artwork_image(
    pool: &PgPool,
    slug: &str,
    from: usize,
    to: usize,
) -> Result<Option<Artwork>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let query = format!(
        "SELECT {ARTWORK_COLUMNS} FROM artworks WHERE slug = $1 AND status = 'draft' FOR UPDATE"
    );
    let Some(mut artwork) = sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .fetch_optional(&mut *transaction)
        .await?
    else {
        return Ok(None);
    };
    if from >= artwork.images.len() || to >= artwork.images.len() {
        return Ok(None);
    }
    let image = artwork.images.remove(from);
    artwork.images.insert(to, image);
    for (position, image) in artwork.images.iter_mut().enumerate() {
        image.position = position as i32;
    }
    let query = format!(
        "UPDATE artworks SET images = $2, updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' RETURNING {ARTWORK_COLUMNS}"
    );
    let artwork = sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .bind(Json(artwork.images))
        .fetch_optional(&mut *transaction)
        .await?;
    transaction.commit().await?;
    Ok(artwork)
}

pub async fn remove_artwork_image(
    pool: &PgPool,
    slug: &str,
    position: usize,
) -> Result<Option<ArtworkImage>, sqlx::Error> {
    let mut transaction = pool.begin().await?;
    let query = format!(
        "SELECT {ARTWORK_COLUMNS} FROM artworks WHERE slug = $1 AND status = 'draft' FOR UPDATE"
    );
    let Some(mut artwork) = sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .fetch_optional(&mut *transaction)
        .await?
    else {
        return Ok(None);
    };
    if position >= artwork.images.len() {
        return Ok(None);
    }
    let removed = artwork.images.remove(position);
    for (position, image) in artwork.images.iter_mut().enumerate() {
        image.position = position as i32;
    }
    sqlx::query(
        "UPDATE artworks SET images = $2, updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft'",
    )
    .bind(slug)
    .bind(Json(artwork.images))
    .execute(&mut *transaction)
    .await?;
    transaction.commit().await?;
    Ok(Some(removed))
}

pub async fn publish_artwork(pool: &PgPool, slug: &str) -> Result<Option<Artwork>, sqlx::Error> {
    let query = format!(
        "UPDATE artworks SET status = 'published', \
             published_at = COALESCE(published_at, CURRENT_TIMESTAMP), \
             updated_at = CURRENT_TIMESTAMP \
         WHERE slug = $1 AND status = 'draft' AND jsonb_array_length(images) > 0 \
         RETURNING {ARTWORK_COLUMNS}"
    );
    sqlx::query_as::<_, Artwork>(&query)
        .bind(slug)
        .fetch_optional(pool)
        .await
}
