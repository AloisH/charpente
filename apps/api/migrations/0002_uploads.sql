-- Uploads are tracked in the DB; bytes live in S3-compatible storage.
-- Flow: POST /uploads records a pending row + returns a presigned PUT,
-- the browser uploads directly, then POST /uploads/{id}/complete confirms.

CREATE TABLE uploads (
    id           uuid PRIMARY KEY,
    owner_id     uuid NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    key          text NOT NULL UNIQUE,
    filename     text NOT NULL,
    content_type text NOT NULL,
    size_bytes   bigint NOT NULL CHECK (size_bytes >= 0),
    state        text NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'complete')),
    created_at   timestamptz NOT NULL DEFAULT now(),
    updated_at   timestamptz NOT NULL DEFAULT now()
);

CREATE TRIGGER uploads_updated_at
    BEFORE UPDATE ON uploads
    FOR EACH ROW EXECUTE FUNCTION set_updated_at();

-- UUIDv7 ids are time-ordered, so (owner_id, id DESC) doubles as the
-- cursor-pagination index.
CREATE INDEX uploads_owner_id_desc ON uploads (owner_id, id DESC);
