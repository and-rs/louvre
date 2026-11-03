use louvre_storage::{
    ArtworkDetails, ArtworkImageInput, add_artwork_image, create_draft, get_artwork,
    list_published_artworks, publish_artwork, run_migrations,
};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
#[ignore = "requires TEST_DATABASE_URL pointing to an isolated PostgreSQL database"]
async fn draft_upload_and_publish_flow_uses_published_visibility() {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .expect("set TEST_DATABASE_URL to run this PostgreSQL integration test");
    let pool = PgPool::connect(&database_url).await.unwrap();
    run_migrations(&pool).await.unwrap();
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let slug = format!("test-artwork-{suffix}");

    let result = async {
        let draft = create_draft(
            &pool,
            &ArtworkDetails {
                title: "Test artwork".to_owned(),
                slug: slug.clone(),
                description: Some("A test piece.".to_owned()),
                collection: Some("Studies".to_owned()),
                medium: Some("Oil on canvas".to_owned()),
                collection_number: Some(12),
            },
        )
        .await
        .unwrap();
        assert_eq!(draft.status, "draft");
        assert_eq!(draft.collection.as_deref(), Some("Studies"));
        assert_eq!(draft.medium.as_deref(), Some("Oil on canvas"));
        assert_eq!(draft.collection_number, Some(12));
        assert!(
            list_published_artworks(&pool)
                .await
                .unwrap()
                .iter()
                .all(|artwork| artwork.slug != slug)
        );

        let with_image = add_artwork_image(
            &pool,
            &slug,
            ArtworkImageInput {
                alt_text: "A blue study",
                s3_key: &format!("artworks/{slug}/image.png"),
                content_type: "image/png",
                upload_id: "upload-1",
                width: 200,
                height: 100,
            },
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(with_image.images.len(), 1);
        assert_eq!(with_image.images[0].alt_text, "A blue study");
        assert_eq!(with_image.images[0].web_versions[0].width, Some(200));
        assert_eq!(with_image.images[0].web_versions[0].height, Some(100));

        let published = publish_artwork(&pool, &slug).await.unwrap().unwrap();
        assert_eq!(published.status, "published");
        assert_eq!(
            list_published_artworks(&pool)
                .await
                .unwrap()
                .iter()
                .find(|artwork| artwork.slug == slug)
                .unwrap()
                .title,
            "Test artwork"
        );
        assert!(publish_artwork(&pool, &slug).await.unwrap().is_none());
        assert_eq!(
            get_artwork(&pool, &slug).await.unwrap().unwrap().images[0].web_versions[0]
                .content_type,
            "image/png"
        );
    }
    .await;

    sqlx::query("DELETE FROM artworks WHERE slug = $1")
        .bind(&slug)
        .execute(&pool)
        .await
        .unwrap();
    result
}
