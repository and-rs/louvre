use crate::templates::{self, PageMetadata};
use axum::{
    Form,
    extract::{Multipart, Path, State, multipart::Field},
    http::{
        HeaderValue, StatusCode,
        header::{CACHE_CONTROL, CONTENT_TYPE, X_CONTENT_TYPE_OPTIONS},
    },
    response::{IntoResponse, Redirect, Response},
};
use axum_login::AuthSession;
use louvre_auth::{Credentials, StudioBackend};
use louvre_storage::{
    Artwork, ArtworkDetails, ArtworkImageInput, PgPool, Storage, StorageError, add_artwork_image,
    create_draft, get_artwork, list_artworks, list_published_artworks, move_artwork_image,
    publish_artwork, remove_artwork_image, update_artwork_image_alt_text, update_draft,
};
use serde::Deserialize;
use std::io::Cursor;
use std::sync::Arc;
use uuid::Uuid;

const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
pub const UPLOAD_BODY_LIMIT: usize = MAX_IMAGE_BYTES + 64 * 1024;

pub struct AppState {
    pub storage: Storage,
    pub database: PgPool,
}

struct UploadedImage {
    bytes: Vec<u8>,
    extension: &'static str,
    content_type: &'static str,
    width: u32,
    height: u32,
}

#[derive(Default)]
struct ArtworkMultipart {
    title: Option<String>,
    slug: Option<String>,
    description: Option<String>,
    collection: Option<String>,
    medium: Option<String>,
    collection_number: Option<String>,
    upload_id: Option<String>,
    image: Option<UploadedImage>,
    image_seen: bool,
    alt_text: Option<String>,
}

struct ArtworkMultipartError {
    form: ArtworkMultipart,
    status: StatusCode,
    message: &'static str,
}

struct ArtworkImageUpload {
    image: UploadedImage,
    alt_text: String,
    upload_id: Uuid,
}

enum ImageStoreError {
    Storage,
    NotDraft,
    Database,
}

#[derive(Deserialize)]
pub struct UpdateArtworkForm {
    title: String,
    slug: String,
    description: String,
    collection: String,
    medium: String,
    collection_number: String,
}

#[derive(Deserialize)]
pub struct ImageMoveForm {
    direction: String,
}

#[derive(Deserialize)]
pub struct UpdateImageAltTextForm {
    alt_text: String,
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
    render_studio_new(
        None,
        templates::StudioNewFields {
            title: "",
            slug: "",
            description: "",
            collection: "",
            medium: "",
            collection_number: "",
            alt_text: "",
        },
        StatusCode::OK,
    )
}

pub async fn studio_create(State(state): State<Arc<AppState>>, multipart: Multipart) -> Response {
    let form = match parse_artwork_multipart(multipart).await {
        Ok(form) => form,
        Err(error) => return error.render_new(),
    };
    let fields = templates::StudioNewFields {
        title: form.title.as_deref().unwrap_or_default().trim(),
        slug: form.slug.as_deref().unwrap_or_default().trim(),
        description: form.description.as_deref().unwrap_or_default().trim(),
        collection: form.collection.as_deref().unwrap_or_default().trim(),
        medium: form.medium.as_deref().unwrap_or_default().trim(),
        collection_number: form.collection_number.as_deref().unwrap_or_default().trim(),
        alt_text: form.alt_text.as_deref().unwrap_or_default().trim(),
    };
    let upload_id = match parse_upload_id(form.upload_id.as_deref()) {
        Ok(upload_id) => upload_id,
        Err(()) => {
            return render_studio_new(
                Some("The image upload identifier is invalid. Please try again."),
                fields,
                StatusCode::BAD_REQUEST,
            );
        }
    };
    let collection_number = match parse_collection_number(fields.collection_number) {
        Some(value) => value,
        None => {
            return render_studio_new(
                Some("Collection number must be a positive whole number."),
                fields,
                StatusCode::BAD_REQUEST,
            );
        }
    };
    if fields.title.is_empty()
        || fields.title.len() > 200
        || !valid_slug(fields.slug)
        || fields.description.len() > 5000
        || fields.collection.len() > 200
        || fields.medium.len() > 200
    {
        return render_studio_new(
            Some("Check the title, URL slug, description, collection, and medium lengths."),
            fields,
            StatusCode::BAD_REQUEST,
        );
    }

    let image_upload = match (form.image, fields.alt_text.is_empty()) {
        (None, true) => None,
        (None, false) => {
            return render_studio_new(
                Some(
                    "Choose an image to use with its alternative text, or leave both fields blank.",
                ),
                fields,
                StatusCode::BAD_REQUEST,
            );
        }
        (Some(_), true) => {
            return render_studio_new(
                Some("Enter alternative text for the image."),
                fields,
                StatusCode::BAD_REQUEST,
            );
        }
        (Some(image), false) => Some(ArtworkImageUpload {
            image,
            alt_text: fields.alt_text.to_owned(),
            upload_id,
        }),
    };

    let details = ArtworkDetails {
        title: fields.title.to_owned(),
        slug: fields.slug.to_owned(),
        description: (!fields.description.is_empty()).then(|| fields.description.to_owned()),
        collection: (!fields.collection.is_empty()).then(|| fields.collection.to_owned()),
        medium: (!fields.medium.is_empty()).then(|| fields.medium.to_owned()),
        collection_number,
    };
    match create_draft(&state.database, &details).await {
        Ok(artwork) => {
            if let Some(image_upload) = image_upload {
                match store_artwork_image(&state, &artwork, image_upload).await {
                    Ok(_) => no_store(
                        Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response(),
                    ),
                    Err(error) => {
                        let (status, message) = image_store_error(&error, true);
                        render_studio_artwork(&artwork, Some(message), status)
                    }
                }
            } else {
                no_store(
                    Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response(),
                )
            }
        }
        Err(error) if is_unique_violation(&error) => render_studio_new(
            Some("That URL slug is already in use. Select the image again if needed."),
            fields,
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

pub async fn studio_thumbnail(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
) -> Response {
    studio_thumbnail_at(&state, &slug, 0).await
}

pub async fn studio_image_thumbnail(
    State(state): State<Arc<AppState>>,
    Path((slug, position)): Path<(String, usize)>,
) -> Response {
    studio_thumbnail_at(&state, &slug, position).await
}

async fn studio_thumbnail_at(state: &AppState, slug: &str, position: usize) -> Response {
    let artwork = match get_artwork(&state.database, slug).await {
        Ok(Some(artwork)) => artwork,
        Ok(None) => return StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load Studio artwork image");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let Some(version) = artwork
        .images
        .get(position)
        .and_then(|image| image.web_versions.first())
    else {
        return StatusCode::NOT_FOUND.into_response();
    };

    match state.storage.get(&version.s3_key).await {
        Ok(bytes) => no_store(
            (
                [
                    (CONTENT_TYPE, version.content_type.as_str()),
                    (X_CONTENT_TYPE_OPTIONS, "nosniff"),
                ],
                bytes,
            )
                .into_response(),
        ),
        Err(StorageError::NotFound) => StatusCode::NOT_FOUND.into_response(),
        Err(StorageError::Other(error)) => {
            tracing::error!(%error, artwork_slug = %slug, position, "failed to fetch Studio artwork image");
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}

pub async fn studio_update(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    Form(form): Form<UpdateArtworkForm>,
) -> Response {
    let title = form.title.trim();
    let new_slug = form.slug.trim();
    let description = form.description.trim();
    let collection = form.collection.trim();
    let medium = form.medium.trim();
    let collection_number = parse_collection_number(form.collection_number.trim());
    let current = match get_artwork(&state.database, &slug).await {
        Ok(Some(artwork)) => artwork,
        Ok(None) => return not_found().await,
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to load artwork before update");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    };
    let render_error = |message, status| {
        let mut displayed = current.clone();
        displayed.title = form.title.clone();
        displayed.slug = form.slug.clone();
        displayed.description = (!description.is_empty()).then_some(form.description.clone());
        displayed.collection = (!collection.is_empty()).then_some(collection.to_owned());
        displayed.medium = (!medium.is_empty()).then_some(medium.to_owned());
        if let Some(collection_number) = collection_number {
            displayed.collection_number = collection_number;
        }
        render_studio_artwork_at(&displayed, Some(message), status, &slug)
    };
    let Some(collection_number) = collection_number else {
        return render_error(
            "Collection number must be a positive whole number.",
            StatusCode::BAD_REQUEST,
        );
    };
    if title.is_empty()
        || title.len() > 200
        || !valid_slug(new_slug)
        || description.len() > 5000
        || collection.len() > 200
        || medium.len() > 200
    {
        return render_error(
            "Check the title, URL slug, description, collection, and medium lengths.",
            StatusCode::BAD_REQUEST,
        );
    }

    let details = ArtworkDetails {
        title: title.to_owned(),
        slug: new_slug.to_owned(),
        description: (!description.is_empty()).then(|| description.to_owned()),
        collection: (!collection.is_empty()).then(|| collection.to_owned()),
        medium: (!medium.is_empty()).then(|| medium.to_owned()),
        collection_number,
    };
    match update_draft(&state.database, &slug, &details).await {
        Ok(Some(artwork)) => {
            no_store(Redirect::to(&format!("/studio/artworks/{}", artwork.slug)).into_response())
        }
        Ok(None) => render_studio_artwork(
            &current,
            Some("Only drafts can be edited."),
            StatusCode::CONFLICT,
        ),
        Err(error) if is_unique_violation(&error) => {
            render_error("That URL slug is already in use.", StatusCode::CONFLICT)
        }
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, "failed to update artwork draft");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_upload_image(
    State(state): State<Arc<AppState>>,
    Path(slug): Path<String>,
    multipart: Multipart,
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

    let form = match parse_artwork_multipart(multipart).await {
        Ok(form) => form,
        Err(error) => return render_studio_artwork(&artwork, Some(error.message), error.status),
    };
    if form.title.is_some()
        || form.slug.is_some()
        || form.description.is_some()
        || form.collection.is_some()
        || form.medium.is_some()
        || form.collection_number.is_some()
    {
        return render_studio_artwork(
            &artwork,
            Some("The upload form contains unexpected fields."),
            StatusCode::BAD_REQUEST,
        );
    }
    let Some(image) = form.image else {
        return render_studio_artwork(
            &artwork,
            Some("Choose an image to upload."),
            StatusCode::BAD_REQUEST,
        );
    };
    let Some(alt_text) = form.alt_text.filter(|value| !value.trim().is_empty()) else {
        return render_studio_artwork(
            &artwork,
            Some("Alternative text is required and must be under 500 characters."),
            StatusCode::BAD_REQUEST,
        );
    };
    let upload_id = match parse_upload_id(form.upload_id.as_deref()) {
        Ok(upload_id) => upload_id,
        Err(()) => {
            return render_studio_artwork(
                &artwork,
                Some("The upload identifier is invalid. Please try again."),
                StatusCode::BAD_REQUEST,
            );
        }
    };
    let upload_id_text = upload_id.to_string();
    if artwork
        .images
        .iter()
        .any(|image| image.upload_id.as_deref() == Some(upload_id_text.as_str()))
    {
        return studio_artwork_redirect(&slug);
    }

    match store_artwork_image(
        &state,
        &artwork,
        ArtworkImageUpload {
            image,
            alt_text: alt_text.trim().to_owned(),
            upload_id,
        },
    )
    .await
    {
        Ok(_) => no_store(Redirect::to(&format!("/studio/artworks/{slug}")).into_response()),
        Err(error) => {
            let (status, message) = image_store_error(&error, false);
            render_studio_artwork(&artwork, Some(message), status)
        }
    }
}

pub async fn studio_update_image_alt_text(
    State(state): State<Arc<AppState>>,
    Path((slug, position)): Path<(String, usize)>,
    Form(form): Form<UpdateImageAltTextForm>,
) -> Response {
    let alt_text = form.alt_text.trim();
    if alt_text.is_empty() || alt_text.len() > 500 {
        return (
            StatusCode::BAD_REQUEST,
            "Alternative text must be 1–500 characters.",
        )
            .into_response();
    }
    match update_artwork_image_alt_text(&state.database, &slug, position, alt_text).await {
        Ok(Some(_)) => studio_artwork_redirect(&slug),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, position, "failed to update image alternative text");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_move_image(
    State(state): State<Arc<AppState>>,
    Path((slug, position)): Path<(String, usize)>,
    Form(form): Form<ImageMoveForm>,
) -> Response {
    let target = match form.direction.as_str() {
        "up" => position.checked_sub(1),
        "down" => position.checked_add(1),
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let Some(target) = target else {
        return StatusCode::BAD_REQUEST.into_response();
    };
    match move_artwork_image(&state.database, &slug, position, target).await {
        Ok(Some(_)) => studio_artwork_redirect(&slug),
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, position, "failed to reorder artwork images");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn studio_delete_image(
    State(state): State<Arc<AppState>>,
    Path((slug, position)): Path<(String, usize)>,
) -> Response {
    match remove_artwork_image(&state.database, &slug, position).await {
        Ok(Some(image)) => {
            let mut keys = image
                .web_versions
                .into_iter()
                .map(|version| version.s3_key)
                .collect::<Vec<_>>();
            keys.push(image.source.s3_key);
            keys.sort_unstable();
            keys.dedup();
            for key in keys {
                if let Err(error) = state.storage.delete(&key).await {
                    tracing::error!(?error, %key, artwork_slug = %slug, "failed to delete removed artwork image from storage");
                }
            }
            studio_artwork_redirect(&slug)
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(error) => {
            tracing::error!(%error, artwork_slug = %slug, position, "failed to remove artwork image");
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
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
    fields: templates::StudioNewFields<'_>,
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
                templates::studio_new(error, fields),
            ),
        )
            .into_response(),
    )
}

impl ArtworkMultipartError {
    fn render_new(self) -> Response {
        render_studio_new(
            Some(self.message),
            templates::StudioNewFields {
                title: self.form.title.as_deref().unwrap_or_default(),
                slug: self.form.slug.as_deref().unwrap_or_default(),
                description: self.form.description.as_deref().unwrap_or_default(),
                collection: self.form.collection.as_deref().unwrap_or_default(),
                medium: self.form.medium.as_deref().unwrap_or_default(),
                collection_number: self.form.collection_number.as_deref().unwrap_or_default(),
                alt_text: self.form.alt_text.as_deref().unwrap_or_default(),
            },
            self.status,
        )
    }
}

async fn parse_artwork_multipart(
    mut multipart: Multipart,
) -> Result<ArtworkMultipart, ArtworkMultipartError> {
    let mut form = ArtworkMultipart::default();
    loop {
        let field = match multipart.next_field().await {
            Ok(Some(field)) => field,
            Ok(None) => return Ok(form),
            Err(error) => {
                tracing::warn!(%error, "invalid multipart artwork form");
                return Err(ArtworkMultipartError {
                    form,
                    status: StatusCode::BAD_REQUEST,
                    message: "The form could not be read. Try again.",
                });
            }
        };
        let Some(name) = field.name().map(str::to_owned) else {
            return Err(ArtworkMultipartError {
                form,
                status: StatusCode::BAD_REQUEST,
                message: "The form was incomplete.",
            });
        };

        let result = match name.as_str() {
            "image" if !form.image_seen => {
                form.image_seen = true;
                parse_image_field(field)
                    .await
                    .map(|image| form.image = image)
            }
            name => {
                let destination = match name {
                    "title" if form.title.is_none() => &mut form.title,
                    "slug" if form.slug.is_none() => &mut form.slug,
                    "description" if form.description.is_none() => &mut form.description,
                    "collection" if form.collection.is_none() => &mut form.collection,
                    "medium" if form.medium.is_none() => &mut form.medium,
                    "collection_number" if form.collection_number.is_none() => {
                        &mut form.collection_number
                    }
                    "upload_id" if form.upload_id.is_none() => &mut form.upload_id,
                    "alt_text" if form.alt_text.is_none() => &mut form.alt_text,
                    _ => {
                        return Err(ArtworkMultipartError {
                            form,
                            status: StatusCode::BAD_REQUEST,
                            message: "The form contains duplicate or unexpected fields.",
                        });
                    }
                };
                let value = if name == "alt_text" {
                    parse_alt_text_field(field).await
                } else {
                    read_text_field(field).await
                };
                value.map(|value| *destination = Some(value))
            }
        };
        if let Err((status, message)) = result {
            return Err(ArtworkMultipartError {
                form,
                status,
                message,
            });
        }
    }
}

async fn read_text_field(field: Field<'_>) -> Result<String, (StatusCode, &'static str)> {
    field.text().await.map_err(|error| {
        tracing::warn!(%error, "failed to read multipart artwork text field");
        (
            StatusCode::BAD_REQUEST,
            "The form could not be read. Try again.",
        )
    })
}

async fn parse_image_field(
    field: Field<'_>,
) -> Result<Option<UploadedImage>, (StatusCode, &'static str)> {
    let file_name = field.file_name().unwrap_or_default().to_owned();
    let bytes = field.bytes().await.map_err(|error| {
        tracing::warn!(%error, "failed to read uploaded image");
        (
            StatusCode::BAD_REQUEST,
            "The uploaded image could not be read. Try again.",
        )
    })?;
    if file_name.is_empty() {
        return Ok(None);
    }
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err((
            StatusCode::PAYLOAD_TOO_LARGE,
            "Choose an image no larger than 10 MB.",
        ));
    }
    let reader = image::ImageReader::new(Cursor::new(&bytes))
        .with_guessed_format()
        .map_err(|error| {
            tracing::warn!(%error, "failed to read uploaded image dimensions");
            (
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Upload a valid JPEG, PNG, or WebP image.",
            )
        })?;
    let (extension, content_type) = match reader.format() {
        Some(image::ImageFormat::Jpeg) => ("jpg", "image/jpeg"),
        Some(image::ImageFormat::Png) => ("png", "image/png"),
        Some(image::ImageFormat::WebP) => ("webp", "image/webp"),
        _ => {
            return Err((
                StatusCode::UNSUPPORTED_MEDIA_TYPE,
                "Upload a valid JPEG, PNG, or WebP image.",
            ));
        }
    };
    let dimensions = reader.into_dimensions().map_err(|error| {
        tracing::warn!(%error, "failed to read uploaded image dimensions");
        (
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "Upload a valid JPEG, PNG, or WebP image.",
        )
    })?;
    Ok(Some(UploadedImage {
        bytes: bytes.to_vec(),
        extension,
        content_type,
        width: dimensions.0,
        height: dimensions.1,
    }))
}

async fn parse_alt_text_field(field: Field<'_>) -> Result<String, (StatusCode, &'static str)> {
    let value = read_text_field(field).await?;
    let value = value.trim();
    if value.len() > 500 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Alternative text must be under 500 characters.",
        ));
    }
    Ok(value.to_owned())
}

async fn store_artwork_image(
    state: &AppState,
    artwork: &Artwork,
    upload: ArtworkImageUpload,
) -> Result<Artwork, ImageStoreError> {
    let file = format!("{}.{}", Uuid::new_v4().simple(), upload.image.extension);
    let key = format!("artworks/{}/{file}", artwork.slug);
    let upload_id = upload.upload_id.to_string();
    let width = upload.image.width;
    let height = upload.image.height;
    if let Err(error) = state
        .storage
        .put(&key, upload.image.bytes, upload.image.content_type)
        .await
    {
        tracing::error!(?error, %key, "failed to store artwork image");
        return Err(ImageStoreError::Storage);
    }

    match add_artwork_image(
        &state.database,
        &artwork.slug,
        ArtworkImageInput {
            alt_text: &upload.alt_text,
            s3_key: &key,
            content_type: upload.image.content_type,
            upload_id: &upload_id,
            width,
            height,
        },
    )
    .await
    {
        Ok(Some(artwork)) => Ok(artwork),
        Ok(None) => {
            delete_orphaned_upload(&state.storage, &key).await;
            match get_artwork(&state.database, &artwork.slug).await {
                Ok(Some(current))
                    if current
                        .images
                        .iter()
                        .any(|image| image.upload_id.as_deref() == Some(upload_id.as_str())) =>
                {
                    Ok(current)
                }
                Ok(_) => Err(ImageStoreError::NotDraft),
                Err(error) => {
                    tracing::error!(%error, artwork_slug = %artwork.slug, "failed to verify a retried image upload");
                    Err(ImageStoreError::Database)
                }
            }
        }
        Err(error) => {
            tracing::error!(%error, artwork_slug = %artwork.slug, "failed to save uploaded image metadata");
            delete_orphaned_upload(&state.storage, &key).await;
            Err(ImageStoreError::Database)
        }
    }
}

fn image_store_error(error: &ImageStoreError, creating: bool) -> (StatusCode, &'static str) {
    match (error, creating) {
        (ImageStoreError::Storage, true) => (
            StatusCode::BAD_GATEWAY,
            "The draft was created, but its image could not be stored. You can upload it below.",
        ),
        (ImageStoreError::Storage, false) => (
            StatusCode::BAD_GATEWAY,
            "The image could not be stored. Check storage configuration and try again.",
        ),
        (ImageStoreError::NotDraft, _) => (
            StatusCode::CONFLICT,
            "This artwork is no longer a draft, so the image was not attached.",
        ),
        (ImageStoreError::Database, true) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The draft was created, but its image could not be attached. You can upload it below.",
        ),
        (ImageStoreError::Database, false) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            "The image uploaded but could not be attached to the draft. Please try again.",
        ),
    }
}

fn render_studio_artwork(artwork: &Artwork, error: Option<&str>, status: StatusCode) -> Response {
    render_studio_artwork_at(artwork, error, status, &artwork.slug)
}

fn render_studio_artwork_at(
    artwork: &Artwork,
    error: Option<&str>,
    status: StatusCode,
    route_slug: &str,
) -> Response {
    no_store(
        (
            status,
            templates::page(
                PageMetadata {
                    page_title: Some(&artwork.title),
                    description: "Manage an artwork in Studio.",
                },
                templates::studio_artwork_at(artwork, error, route_slug),
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
                description: artwork.description.as_deref().unwrap_or(&format!(
                    "Artwork from the {} collection.",
                    crate::branding::site_name()
                )),
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
    let Some(version) = artwork
        .images
        .iter()
        .flat_map(|image| image.web_versions.iter())
        .find(|version| version.s3_key.rsplit('/').next() == Some(file.as_str()))
    else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let key = &version.s3_key;

    match state.storage.get(key).await {
        Ok(bytes) => (
            [
                (CONTENT_TYPE, version.content_type.as_str()),
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

fn studio_artwork_redirect(slug: &str) -> Response {
    no_store(Redirect::to(&format!("/studio/artworks/{slug}")).into_response())
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

fn parse_collection_number(value: &str) -> Option<Option<i32>> {
    if value.is_empty() {
        Some(None)
    } else {
        value
            .parse::<i32>()
            .ok()
            .filter(|number| *number > 0)
            .map(Some)
    }
}

fn parse_upload_id(value: Option<&str>) -> Result<Uuid, ()> {
    match value.filter(|value| !value.is_empty()) {
        Some(value) => Uuid::parse_str(value).map_err(|_| ()),
        None => Ok(Uuid::new_v4()),
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
    use axum::{body::Body, extract::FromRequest, http::Request};

    async fn parse_test_form(
        fields: &[(&str, Option<&str>, &[u8])],
    ) -> Result<ArtworkMultipart, ArtworkMultipartError> {
        let mut body = Vec::new();
        for (name, filename, bytes) in fields {
            body.extend_from_slice(
                format!("--test-boundary\r\nContent-Disposition: form-data; name=\"{name}\"")
                    .as_bytes(),
            );
            if let Some(filename) = filename {
                body.extend_from_slice(format!("; filename=\"{filename}\"").as_bytes());
            }
            body.extend_from_slice(b"\r\n\r\n");
            body.extend_from_slice(bytes);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(b"--test-boundary--\r\n");
        let request = Request::post("/")
            .header(CONTENT_TYPE, "multipart/form-data; boundary=test-boundary")
            .body(Body::from(body))
            .unwrap();
        let multipart = Multipart::from_request(request, &()).await.unwrap();
        parse_artwork_multipart(multipart).await
    }

    #[test]
    fn validates_public_slugs() {
        assert!(valid_slug("quiet-study-2"));
        assert!(!valid_slug("Quiet-study"));
        assert!(!valid_slug("-quiet"));
        assert!(!valid_slug("quiet--study"));
        assert!(!valid_slug("quiet/study"));
    }

    #[tokio::test]
    async fn multipart_reads_text_fields_and_accepts_an_empty_optional_image() {
        let form = parse_test_form(&[
            ("title", None, b"Study"),
            ("slug", None, b"study"),
            ("description", None, b"Description"),
            ("collection", None, b"Studies"),
            ("medium", None, b"Oil"),
            ("collection_number", None, b"12"),
            ("upload_id", None, b"client-id"),
            ("alt_text", None, b"  Alternative text  "),
            ("image", Some(""), b""),
        ])
        .await
        .unwrap_or_else(|error| panic!("{}", error.message));

        assert_eq!(form.title.as_deref(), Some("Study"));
        assert_eq!(form.slug.as_deref(), Some("study"));
        assert_eq!(form.description.as_deref(), Some("Description"));
        assert_eq!(form.collection.as_deref(), Some("Studies"));
        assert_eq!(form.medium.as_deref(), Some("Oil"));
        assert_eq!(form.collection_number.as_deref(), Some("12"));
        assert_eq!(form.upload_id.as_deref(), Some("client-id"));
        assert_eq!(form.alt_text.as_deref(), Some("Alternative text"));
        assert!(form.image_seen);
        assert!(form.image.is_none());
    }

    #[tokio::test]
    async fn multipart_rejects_duplicates_and_unknown_fields_without_losing_entered_values() {
        for name in [
            "title",
            "slug",
            "description",
            "collection",
            "medium",
            "collection_number",
            "upload_id",
            "alt_text",
            "image",
            "unexpected",
        ] {
            let mut fields = vec![("title", None, b"Saved title".as_slice())];
            if name != "title" && name != "unexpected" {
                fields.push((name, Some(""), b""));
            }
            fields.push((name, Some(""), b""));
            let error = parse_test_form(&fields).await.err().expect(name);
            assert_eq!(error.status, StatusCode::BAD_REQUEST, "{name}");
            assert_eq!(
                error.message,
                "The form contains duplicate or unexpected fields."
            );
            assert_eq!(error.form.title.as_deref(), Some("Saved title"), "{name}");
        }
    }

    #[tokio::test]
    async fn uploaded_images_use_detected_formats_and_dimensions() {
        for (format, extension, content_type) in [
            (image::ImageFormat::Jpeg, "jpg", "image/jpeg"),
            (image::ImageFormat::Png, "png", "image/png"),
            (image::ImageFormat::WebP, "webp", "image/webp"),
        ] {
            let mut bytes = Cursor::new(Vec::new());
            image::DynamicImage::new_rgb8(2, 3)
                .write_to(&mut bytes, format)
                .unwrap();
            let form = parse_test_form(&[("image", Some("misleading.gif"), bytes.get_ref())])
                .await
                .unwrap_or_else(|error| panic!("{}", error.message));
            let image = form.image.unwrap();
            assert_eq!(image.extension, extension);
            assert_eq!(image.content_type, content_type);
            assert_eq!((image.width, image.height), (2, 3));
            assert_eq!(image.bytes, bytes.into_inner());
        }
    }

    #[tokio::test]
    async fn multipart_preserves_values_when_image_or_alt_text_validation_fails() {
        for bytes in [
            b"not an image".as_slice(),
            b"GIF89a",
            b"\xff\xd8\xff",
            b"\x89PNG\r\n\x1a\n",
            b"RIFFxxxxWEBP",
        ] {
            let error = parse_test_form(&[
                ("title", None, b"Saved title"),
                ("image", Some("image.png"), bytes),
            ])
            .await
            .err()
            .expect("invalid image must fail");
            assert_eq!(error.status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
            assert_eq!(error.form.title.as_deref(), Some("Saved title"));
        }
        let long_alt_text = "x".repeat(501);
        let error = parse_test_form(&[
            ("title", None, b"Saved title"),
            ("alt_text", None, long_alt_text.as_bytes()),
        ])
        .await
        .err()
        .expect("oversized alt text must fail");
        assert_eq!(error.status, StatusCode::BAD_REQUEST);
        assert_eq!(error.form.title.as_deref(), Some("Saved title"));
    }

    #[test]
    fn accepts_empty_or_valid_collection_numbers_only() {
        assert_eq!(parse_collection_number(""), Some(None));
        assert_eq!(parse_collection_number("42"), Some(Some(42)));
        assert_eq!(parse_collection_number("0"), None);
        assert_eq!(parse_collection_number("-2"), None);
        assert_eq!(parse_collection_number("42.5"), None);
        assert_eq!(parse_collection_number("2147483648"), None);
    }

    #[test]
    fn parses_client_upload_ids_and_generates_missing_ids() {
        let id = Uuid::new_v4();
        assert_eq!(parse_upload_id(Some(&id.to_string())), Ok(id));
        assert!(parse_upload_id(Some("invalid")).is_err());
        assert!(parse_upload_id(None).is_ok());
    }
}
