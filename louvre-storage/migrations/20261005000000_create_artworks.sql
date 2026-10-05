CREATE TABLE IF NOT EXISTS artworks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slug TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    description TEXT,
    collection TEXT,
    medium TEXT,
    collection_number INTEGER,
    status TEXT NOT NULL DEFAULT 'draft'
        CHECK (status IN ('draft', 'published', 'archived')),
    images JSONB NOT NULL DEFAULT '[]'::jsonb
        CHECK (jsonb_typeof(images) = 'array'),
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    published_at TIMESTAMPTZ
);

COMMENT ON COLUMN artworks.images IS
    'Ordered artwork images with alt_text, position, source.s3_key, and web_versions containing s3_key, content_type, width, and height.';
