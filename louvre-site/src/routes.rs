use crate::templates::{self, PageMetadata};
use axum::{
    Form,
    extract::{Multipart, Path, State},
    http::{
        HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
    },
    response::{IntoResponse, Redirect, Response},
};
use axum_login::AuthSession;
use louvre_auth::{Credentials, StudioBackend};
use louvre_storage::{
    Artwork, PgPool, Storage, StorageError, add_artwork_image, create_draft, get_artwork,
    list_artworks, list_published_artworks, publish_artwork, update_draft,
};
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
pub const UPLOAD_BODY_LIMIT: usize = MAX_IMAGE_BYTES + 64 * 1024;

pub struct AppState {
    pub storage: Storage,
    pub database: PgPool,
}

#[derive(Deserialize)]
pub struct CreateArtworkForm {
    title: String,
    slug: String,
    description: String,
}

#[derive(Deserialize)]
pub struct UpdateArtworkForm {
    title: String,
    description: String,
}

pub async fn studio_login(auth_session: AuthSession<StudioBackend>) -> Response {
    if auth_session.user.is_some() {
        return no_store(Redirect::to("/studio").into_response());
    }

    render_studio_login(None)
}

pub async fn studio_login_submit(
    mut auth_session: AuthSession<StudioBackend>,
    Form(credentials): Form<Credentials>,
) -> Response {
    match auth_session.authenticate(credentials).await {
        Ok(Some(user)) => match auth_session.login(&user).await {
            Ok(()) => no_store(Redirect::to("/studio").into_response()),
            Err(error) => {
                tracing::error!(%error, "failed to create studio session");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Ok(None) => no_store(
            (
                StatusCode::UNAUTHORIZED,
                templates::page(
                    PageMetadata {
                        page_title: Some("Studio sign in"),
                        description: "Sign in to the private artwork workspace.",
                    },
                    templates::studio_login(Some("Those details did not match.")),
                ),
            )
                .into_response(),
        ),
        Err(error) => {
            tracing::error!(%error, "studio authentication failed");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio(
    State(state): State<Arc<AppState>>,
    auth_session: AuthSession<StudioBackend>,
) -> Response {
    let Some(user) = auth_session.user else {
        return no_store(Redirect::to("/studio/login").into_response());
    };
    let artworks = match list_artworks(&state.database).await {
        Ok(artworks) => artworks,
        Err(error) => {
            tracing::error!(%error, "failed to list Studio artwork");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    no_store(
        templates::page(
            PageMetadata {
                page_title: Some("Studio"),
                description: "Private artwork management workspace.",
            },
            templates::studio(&user.username, &artworks),
        )
        .into_response(),
    )
}

pub async fn studio_logout(mut auth_session: AuthSession<StudioBackend>) -> Response {
    match auth_session.logout().await {
        Ok(_) => no_store(Redirect::to("/studio/login").into_response()),
        Err(error) => {
            tracing::error!(%error, "failed to end studio session");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_new() -> Response {
    render_studio_new(None, "", "", "", StatusCode::OK)
}

pub async fn studio_create(
    State(state): State<Arc<AppState>>,
    Form(form): Form<CreateArtworkForm>,
) -> Response {
    let title = form.title.trim();
    let slug = form.slug.trim();
    let description = form.description.trim();
    if title.is_empty() || title.len() > 200 || !valid_slug(slug) || description.len() > 5000 {
        return render_studio_new(
            Some(
                "Enter a title (up to 200 characters), a valid lowercase slug, and a description under 5,000 characters.",
            ),
            &form.title,
            &form.slug,
            &form.description,
            StatusCode::BAD_REQUEST,
        );
    }

    match create_draft(
        &state.database,
        title,
        slug,
        (!description.is_empty()).then_some(description),
    )
    .await
    {
        Ok(artwork) => {
            no_store(Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response())
        }
        Err(error) if is_unique_violation(&error) => render_studio_new(
            Some("That URL slug is already in use."),
            &form.title,
            &form.slug,
            &form.description,
            StatusCode::CONFLICT,
        ),
        Err(error) => {
            tracing::error!(%error, "failed to create artwork draft");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_artwork(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Response {
    match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) => render_studio_artwork(&artwork, None, StatusCode::OK),
        Ok(None) => not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load Studio artwork");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_update(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Form(form): Form<UpdateArtworkForm>,
) -> Response {
    let title = form.title.trim();
    let description = form.description.trim();
    let current = match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) => artwork,
        Ok(None) => return not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load artwork before update");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    if title.is_empty() || title.len() > 200 || description.len() > 5000 {
        let mut displayed = current;
        displayed.title = form.title;
        displayed.description = (!form.description.trim().is_empty()).then_some(form.description);
        return render_studio_artwork(
            &displayed,
            Some("Enter a title under 200 characters and a description under 5,000 characters."),
            StatusCode::BAD_REQUEST,
        );
    }

    match update_draft(
        &state.database,
        &slug,
        title,
        (!description.is_empty()).then_some(description),
    )
    .await
    {
        Ok(Some(artwork)) => {
            no_store(Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response())
        }
        Ok(None) => render_studio_artwork(
            &current,
            Some("Only drafts can be edited."),
            StatusCode::CONFLICT,
        ),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to update artwork draft");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_upload_image(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    mut multipart: Multipart,
) -> Response {
    let artwork = match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) if artwork.status == "draft" => artwork,
        Ok(Some(_)) => return StatusCode::CONFLICT.into_response(),
        Ok(None) => return not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load draft for upload");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };

    let mut image: Option<(Vec<u8>, &'static str, &'static str)> = None;
    let mut alt_text: Option<String> = None;
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => break,
            Err(error) => {
                tracing::warn!(%error, "invalid multipart artwork upload");
                return render_studio_artwork(
                    &artwork,
                    Some("The upload could not be read. Try again with an image under 10 MB."),
                    StatusCode::BAD_REQUEST,
                );
            }
        };
        let Some(name) = field.name().map(str::to_owned) else {
            return render_studio_artwork(
                &artwork,
                Some("The upload form was incomplete."),
                StatusCode::BAD_REQUEST,
            );
        };
        match name.as_str() {
            "image" if image.is_none() => {
                let bytes = match field.bytes().await {
                    Ok(bytes) if !bytes.is_empty() && bytes.len() <= MAX_IMAGE_BYTES => bytes,
                    Ok(_) => {
                        return render_studio_artwork(
                            &artwork,
                            Some("Choose an image no larger than 10 MB."),
                            StatusCode::PAYLOAD_TOO_LARGE,
                        );
                    }
                    Err(error) => {
                        tracing::warn!(%error, "failed to read uploaded image");
                        return render_studio_artwork(
                            &artwork,
                            Some("The uploaded image could not be read."),
                            StatusCode::BAD_REQUEST,
                        );
                    }
                };
                let Some((extension, content_type)) = image_type(&bytes) else {
                    return render_studio_artwork(
                        &artwork,
                        Some("Upload a valid JPEG, PNG, or WebP image."),
                        StatusCode::UNSUPPORTED_MEDIA_TYPE,
                    );
                };
                image = Some((bytes.to_vec(), extension, content_type));
            }
            "alt_text" if alt_text.is_none() => {
                let value = match field.text().await {
                    Ok(value) => value.trim().to_owned(),
                    Err(error) => {
                        tracing::warn!(%error, "failed to read image alternative text");
                        return render_studio_artwork(
                            &artwork,
                            Some("Enter alternative text and try again."),
                            StatusCode::BAD_REQUEST,
                        );
                    }
                };
                if value.is_empty() || value.len() > 500 {
                    return render_studio_artwork(
                        &artwork,
                        Some("Alternative text is required and must be under 500 characters."),
                        StatusCode::BAD_REQUEST,
                    );
                }
                alt_text = Some(value);
            }
            _ => {
                return render_studio_artwork(
                    &artwork,
                    Some("The upload form must contain one image and alternative text."),
                    StatusCode::BAD_REQUEST,
                );
            }
        }
    }

    let Some((bytes, extension, content_type)) = image else {
        return render_studio_artwork(
            &artwork,
            Some("Choose an image to upload."),
            StatusCode::BAD_REQUEST,
        );
    };
    let Some(alt_text) = alt_text else {
        return render_studio_artwork(
            &artwork,
            Some("Alternative text is required."),
            StatusCode::BAD_REQUEST,
        );
    };
    let file = format!("{}.{}", Uuid::new_v4().simple(), extension);
    let key = format!("artworks/{}/{file}", artwork.slug);
    if let Err(error) = state.storage.put(&key, bytes, content_type).await {
        tracing::error!(?error, %key, "failed to upload artwork image to S3");
        return render_studio_artwork(
            &artwork,
            Some("The image could not be stored. Check S3 configuration and try again."),
            StatusCode::BAD_GATEWAY,
        );
    }

    match add_artwork_image(&state.database, &slug, &alt_text, &key, content_type).await {
        Ok(Some(_)) => no_store(Redirect::to(&format!("/studio/artworks/{slug}")).into_response()),
        Ok(None) => {
            delete_orphaned_upload(&state.storage, &key).await;
            render_studio_artwork(
                &artwork,
                Some("This artwork is no longer a draft, so the image was not attached."),
                StatusCode::CONFLICT,
            )
        }
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to save uploaded image metadata");
            delete_orphaned_upload(&state.storage, &key).await;
            render_studio_artwork(
                &artwork,
                Some(
                    "The image uploaded but could not be attached to the draft. Please try again.",
                ),
                StatusCode::INTERNAL_SERVER_ERROR,
            )
        }
    }
}

pub async fn studio_publish(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Response {
    match publish_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) => {
            no_store(Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response())
        }
        Ok(None) => match get_artwork(&state.database, &slug).await {
            Ok(Some(artwork)) if artwork.images.is_empty() => render_studio_artwork(
                &artwork,
                Some("Upload at least one image before publishing."),
                StatusCode::BAD_REQUEST,
            ),
            Ok(Some(artwork)) => render_studio_artwork(
                &artwork,
                Some("Only drafts can be published."),
                StatusCode::CONFLICT,
            ),
            Ok(None) => not_found().await,
            Err(error) => {
                tracing::error!(%error, artwork_slug = %slug, "failed to check artwork after publish");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        },
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to publish artwork");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn render_studio_login(error: Option<&str>) -> Response {
    no_store(
        templates::page(
            PageMetadata {
                page_title: Some("Studio sign in"),
                description: "Sign in to the private artwork workspace.",
            },
            templates::studio_login(error),
        )
        .into_response(),
    )
}

fn render_studio_new(
    error: Option<&str>,
    title: &str,
    slug: &str,
    description: &str,
    status: StatusCode,
) -> Response {
    no_store(
        (
            status,
            templates::page(
                PageMetadata {
                    page_title: Some("New artwork"),
                    description: "Create an artwork draft in Studio.",
                },
                templates::studio_new(error, title, slug, description),
            ),
        )
            .into_response(),
    )
}

fn render_studio_artwork(artwork: &Artwork, error: Option<&str>, status: StatusCode) -> Response {
    no_store(
        (
            status,
            templates::page(
                PageMetadata {
                    page_title: Some(&artwork.title),
                    description: "Manage an artwork in Studio.",
                },
                templates::studio_artwork(artwork, error),
            ),
        )
            .into_response(),
    )
}

fn no_store(mut response: Response) -> Response {
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("private, no-store"));
    response
}

pub async fn home(State(state): State<Arc<AppState>>) -> Response {
    let artworks = match list_published_artworks(&state.database).await {
        Ok(artworks) => artworks,
        Err(error) => {
            tracing::error!(%error, "failed to load public artwork collection");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    templates::page(
        PageMetadata {
            page_title: None,
            description: "An artwork publishing platform.",
        },
        templates::home(&artworks),
    )
    .into_response()
}

pub async fn health(State(state): State<Arc<AppState>>) -> StatusCode {
    match state.database.acquire().await {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            tracing::warn!(%error, "failed to get PostgreSQL connection for health check");
            StatusCode::SERVICE_UNAVAILABLE
        }
    }
}

pub async fn artwork(State(state): State<Arc<AppState>>, Path(slug): Path<String>) -> Response {
    match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) if artwork.status == "published" => templates::page(
            PageMetadata {
                page_title: Some(&artwork.title),
                description: artwork
                    .description
                    .as_deref()
                    .unwrap_or("Artwork from the Louvre collection."),
            },
            templates::artwork(&artwork),
        )
        .into_response(),
        Ok(_) => not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load public artwork");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn artwork_image(
    State(state): State<Arc<AppState>>,
    Path((slug, file)): Path<(String, String)>,
) -> Response {
    if !valid_slug(&slug)
        || file.is_empty()
        || !file.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'.' || byte == b'_'
        })
    {
        return StatusCode::BAD_REQUEST.into_response();
    }
    let artwork = match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) if artwork.status == "published" => artwork,
        Ok(_) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load artwork for image request");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let key = format!("artworks/{slug}/{file}");
    let content_type = artwork
        .images
        .iter()
        .flat_map(|image| image.web_versions.iter())
        .find(|version| version.s3_key == key)
        .map(|version| version.content_type.as_str());
    let Some(content_type) = content_type else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match state.storage.get(&key).await {
        Ok(bytes) => (
            [
                (CONTENT_TYPE, content_type),
                (CACHE_CONTROL, "public, max-age=31536000, immutable"),
                (X_CONTENT_TYPE_OPTIONS, "nosniff"),
            ],
            bytes,
        )
            .into_response(),
        Err(StorageError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(StorageError::Other(error)) => {
            tracing::warn!(%error, %key, "failed to fetch artwork image");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn not_found() -> Response {
    (
        StatusCode::NOT_FOUND,
        templates::page(
            PageMetadata {
                page_title: Some("Not found"),
                description: "The requested page does not exist.",
            },
            templates::not_found(),
        ),
    )
        .into_response()
}

fn valid_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug.len() <= 100
        && !slug.starts_with('-')
        && !slug.ends_with('-')
        && !slug.contains("--")
        && slug
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

fn image_type(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    if bytes.starts_with(&[0xff, 0xd8, 0xff]) {
        Some(("jpg", "image/jpeg"))
    } else if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        Some(("png", "image/png"))
    } else if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(("webp", "image/webp"))
    } else {
        None
    }
}

fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(database_error) if database_error.code().as_deref() == Some("23505"))
}

async fn delete_orphaned_upload(storage: &Storage, key: &str) {
    if let Err(error) = storage.delete(key).await {
        tracing::error!(?error, %key, "failed to remove unattached artwork image from S3");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_public_slugs() {
        assert!(valid_slug("quiet-study-2"));
        assert!(!valid_slug("Quiet-study"));
        assert!(!valid_slug("-quiet"));
        assert!(!valid_slug("quiet--study"));
        assert!(!valid_slug("quiet/study"));
    }

    #[test]
    fn detects_supported_image_signatures() {
        assert_eq!(image_type(&[0xff, 0xd8, 0xff]), Some(("jpg", "image/jpeg")));
        assert_eq!(
            image_type(b"\x89PNG\r\n\x1a\nrest"),
            Some(("png", "image/png"))
        );
        assert_eq!(
            image_type(b"RIFFxxxxWEBPdata"),
            Some(("webp", "image/webp"))
        );
        assert_eq!(image_type(b"not an image"), None);
    }
}
