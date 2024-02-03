-- Add migration script here
CREATE TABLE IF NOT EXISTS message (
    original_message_id BIGINT NOT NULL,
    id BIGINT PRIMARY KEY NOT NULL,
    channel_id BIGINT NOT NULL
);