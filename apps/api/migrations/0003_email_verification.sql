-- Email verification: nullable timestamp on users (NULL = unverified), and an
-- index so re-issuing tokens can clear a user's previous ones cheaply.

ALTER TABLE users ADD COLUMN email_verified_at timestamptz;

CREATE INDEX email_verification_tokens_user
    ON email_verification_tokens (user_id);
