use std::{
    fs::File,
    io::{self, Cursor, Read},
};

use image::{ImageFormat, ImageReader};
use louvre_storage::{
    Artwork, ArtworkDetails, ArtworkImageInput, PgPool, Storage, add_artwork_image, create_draft,
    get_artwork, list_artworks, move_artwork_image, publish_artwork, remove_artwork_image,
    update_artwork_image_alt_text, update_draft, valid_slug,
};
use sqlx::{postgres::PgConnectOptions, postgres::PgPoolOptions};
use uuid::Uuid;

use crate::Result;

const LOCAL_DATABASE: &str = "postgres://louvre@127.0.0.1:55432/louvre";
const MAX_IMAGE_BYTES: usize = 10 * 1024 * 1024;
pub const USAGE: &str = "\
Usage: louvre artwork [--railway] <command>
  list
  show <slug>
  new <slug> <title> [--description TEXT] [--collection TEXT] [--medium TEXT] [--number N]
  edit <slug> [--slug SLUG] [--title TEXT] [--description TEXT] [--collection TEXT] [--medium TEXT] [--number N]
  add-image <slug> <file|-> <alt-text>
  alt-text <slug> <position> <alt-text>
  move-image <slug> <from> <to>
  remove-image <slug> <position>
  publish <slug>

Image positions start at 1. Use '-' to read an image from stdin.
--railway is required inside a Railway service; otherwise writes are local.";

pub async fn run(args: &[&str]) -> Result<()> {
    let (remote, args) = match args {
        ["--railway", rest @ ..] => (true, rest),
        _ => (false, args),
    };
    if matches!(args, [] | ["-h" | "--help" | "help"]) {
        println!("{USAGE}");
        return Ok(());
    }
    let slug = match args {
        ["list"] => None,
        ["show" | "publish", slug]
        | ["new", slug, _, ..]
        | ["edit", slug, _, ..]
        | ["add-image" | "alt-text" | "move-image", slug, _, _]
        | ["remove-image", slug, _] => Some(*slug),
        _ => return Err(invalid(USAGE).into()),
    };
    if let Some(slug) = slug {
        check_slug(slug)?;
    }
    let on_railway = std::env::var_os("RAILWAY_ENVIRONMENT_ID").is_some_and(|id| !id.is_empty());
    if remote != on_railway {
        return Err(invalid(if remote {
            "--railway must run inside the intended Railway app service"
        } else {
            "inside Railway, explicitly pass artwork --railway"
        })
        .into());
    }
    let url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(std::env::VarError::NotPresent) if !remote => LOCAL_DATABASE.to_owned(),
        Err(_) => return Err(invalid("DATABASE_URL must be set").into()),
    };
    let options: PgConnectOptions = url.parse()?;
    if !remote
        && (options.get_socket().is_some()
            || !matches!(options.get_host(), "127.0.0.1" | "localhost" | "::1"))
    {
        return Err(invalid("local artwork commands require loopback PostgreSQL").into());
    }
    let pool = PgPoolOptions::new().connect_with(options).await?;
    match args {
        ["list"] => {
            for artwork in list_artworks(&pool).await? {
                println!("{}\t{}\t{}", artwork.slug, artwork.status, artwork.title);
            }
        }
        ["show", slug] => {
            let artwork = find(&pool, slug).await?;
            println!("{} [{}] — {}", artwork.slug, artwork.status, artwork.title);
            if let Some(description) = artwork.description {
                println!("{description}");
            }
            for (label, value) in [
                ("Collection", artwork.collection),
                ("Medium", artwork.medium),
            ] {
                if let Some(value) = value {
                    println!("{label}: {value}");
                }
            }
            if let Some(number) = artwork.collection_number {
                println!("Number: {number}");
            }
            for image in artwork.images {
                println!(
                    "{}\t{}\t{}",
                    image.position + 1,
                    image.alt_text,
                    image.source.s3_key
                );
            }
        }
        ["new", slug, title, flags @ ..] => {
            let details = details(
                ArtworkDetails {
                    slug: (*slug).into(),
                    title: (*title).into(),
                    description: None,
                    collection: None,
                    medium: None,
                    collection_number: None,
                },
                flags,
                false,
            )?;
            output(create_draft(&pool, &details).await?);
        }
        ["edit", slug, flags @ ..] => {
            let current = draft(&pool, slug).await?;
            let details = details(
                ArtworkDetails {
                    slug: current.slug,
                    title: current.title,
                    description: current.description,
                    collection: current.collection,
                    medium: current.medium,
                    collection_number: current.collection_number,
                },
                flags,
                true,
            )?;
            output(
                update_draft(&pool, slug, &details)
                    .await?
                    .ok_or_else(not_draft)?,
            );
        }
        ["add-image", slug, file, alt] => {
            draft(&pool, slug).await?;
            let alt = alt_text(alt)?;
            let bytes = if *file == "-" {
                read_image(io::stdin().lock())?
            } else {
                read_image(File::open(file)?)?
            };
            let (extension, content_type, width, height) = image_details(&bytes)?;
            let upload_id = Uuid::new_v4().simple().to_string();
            let key = format!("artworks/{slug}/{upload_id}.{extension}");
            let storage = storage(remote).await;
            storage
                .put(&key, bytes, content_type)
                .await
                .map_err(storage_error)?;
            match add_artwork_image(
                &pool,
                slug,
                ArtworkImageInput {
                    alt_text: alt,
                    s3_key: &key,
                    content_type,
                    upload_id: &upload_id,
                    width,
                    height,
                },
            )
            .await
            {
                Ok(Some(artwork)) => output(artwork),
                Ok(None) => {
                    storage.delete(&key).await.map_err(storage_error)?;
                    return Err(not_draft().into());
                }
                // A lost database response can mean the write committed. Do not delete a
                // potentially referenced object; print the key so the operator can check.
                Err(error) => return Err(io::Error::other(format!(
                    "could not confirm image attachment ({key}): {error}; inspect the draft before retrying"
                )).into()),
            }
        }
        ["alt-text", slug, at, alt] => {
            output(
                update_artwork_image_alt_text(&pool, slug, position(at)?, alt_text(alt)?)
                    .await?
                    .ok_or_else(bad_image)?,
            );
        }
        ["move-image", slug, from, to] => {
            output(
                move_artwork_image(&pool, slug, position(from)?, position(to)?)
                    .await?
                    .ok_or_else(bad_image)?,
            );
        }
        ["remove-image", slug, at] => {
            let image = remove_artwork_image(&pool, slug, position(at)?)
                .await?
                .ok_or_else(bad_image)?;
            let mut keys = vec![image.source.s3_key];
            keys.extend(image.web_versions.into_iter().map(|version| version.s3_key));
            keys.sort_unstable();
            keys.dedup();
            let storage = storage(remote).await;
            for key in keys {
                storage.delete(&key).await.map_err(|error| {
                    io::Error::other(format!(
                        "image detached, but could not delete {key}: {error:?}"
                    ))
                })?;
            }
            println!("Removed image {at} from {slug}");
        }
        ["publish", slug] => {
            output(publish_artwork(&pool, slug).await?.ok_or_else(|| {
                invalid("publishing requires an existing draft with at least one image")
            })?);
        }
        _ => unreachable!("command shape checked above"),
    }
    Ok(())
}

async fn storage(remote: bool) -> Storage {
    let storage = Storage::from_env().await;
    if remote {
        storage
    } else {
        storage.with_local_uploads(
            std::env::var_os("DEV_UPLOADS_DIR").unwrap_or_else(|| ".local/uploads".into()),
        )
    }
}

async fn find(pool: &PgPool, slug: &str) -> Result<Artwork> {
    Ok(get_artwork(pool, slug)
        .await?
        .ok_or_else(|| invalid("artwork not found"))?)
}

async fn draft(pool: &PgPool, slug: &str) -> Result<Artwork> {
    let artwork = find(pool, slug).await?;
    if artwork.status != "draft" {
        return Err(not_draft().into());
    }
    Ok(artwork)
}

fn output(artwork: Artwork) {
    println!("{} [{}]", artwork.slug, artwork.status);
}

fn details(
    mut details: ArtworkDetails,
    flags: &[&str],
    editing: bool,
) -> io::Result<ArtworkDetails> {
    for pair in flags.chunks(2) {
        let [flag, value] = pair else {
            return Err(invalid("metadata options require a value"));
        };
        let value = value.trim();
        let optional = || (!value.is_empty()).then(|| value.to_owned());
        match *flag {
            "--slug" if editing => details.slug = value.into(),
            "--title" if editing => details.title = value.into(),
            "--description" => details.description = optional(),
            "--collection" => details.collection = optional(),
            "--medium" => details.medium = optional(),
            "--number" => {
                details.collection_number =
                    if value.is_empty() {
                        None
                    } else {
                        Some(value.parse().ok().filter(|&n| n > 0).ok_or_else(|| {
                            invalid("collection number must be a positive integer")
                        })?)
                    };
            }
            _ => return Err(invalid(format!("unknown metadata option: {flag}"))),
        }
    }
    check_slug(&details.slug)?;
    details.title = details.title.trim().to_owned();
    if details.title.is_empty() {
        return Err(invalid("title cannot be empty"));
    }
    for (field, value, limit) in [
        ("title", Some(details.title.as_str()), 200),
        ("description", details.description.as_deref(), 5000),
        ("collection", details.collection.as_deref(), 200),
        ("medium", details.medium.as_deref(), 200),
    ] {
        if value.is_some_and(|value| value.len() > limit) {
            return Err(invalid(format!("{field} must be at most {limit} bytes")));
        }
    }
    Ok(details)
}

fn check_slug(slug: &str) -> io::Result<()> {
    if !valid_slug(slug) {
        return Err(invalid(
            "slug must be 1–100 lowercase letters, digits, or single hyphens between words",
        ));
    }
    Ok(())
}

fn position(value: &str) -> io::Result<usize> {
    value
        .parse::<i32>()
        .ok()
        .filter(|&n| n > 0)
        .map(|n| n as usize - 1)
        .ok_or_else(|| invalid("image position must be a positive integer"))
}

fn alt_text(value: &str) -> io::Result<&str> {
    let value = value.trim();
    if value.is_empty() || value.len() > 500 {
        return Err(invalid("alt text must be 1–500 bytes"));
    }
    Ok(value)
}

fn read_image(source: impl Read) -> io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    source
        .take(MAX_IMAGE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes.is_empty() || bytes.len() > MAX_IMAGE_BYTES {
        return Err(invalid("image must be nonempty and at most 10 MiB"));
    }
    Ok(bytes)
}

fn image_details(bytes: &[u8]) -> Result<(&'static str, &'static str, u32, u32)> {
    let reader = ImageReader::new(Cursor::new(bytes)).with_guessed_format()?;
    let (extension, content_type) = match reader.format() {
        Some(ImageFormat::Jpeg) => ("jpg", "image/jpeg"),
        Some(ImageFormat::Png) => ("png", "image/png"),
        Some(ImageFormat::WebP) => ("webp", "image/webp"),
        _ => return Err(invalid("expected a JPEG, PNG, or WebP image").into()),
    };
    let image = reader.decode()?;
    Ok((extension, content_type, image.width(), image.height()))
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidInput, message.into())
}

fn not_draft() -> io::Error {
    invalid("artwork is not a draft")
}

fn bad_image() -> io::Error {
    invalid("no image at that position on an editable draft")
}

fn storage_error(error: louvre_storage::StorageError) -> io::Error {
    io::Error::other(format!("image storage: {error:?}"))
}
