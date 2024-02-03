-- Add migration script here
CREATE TABLE IF NOT EXISTS message (
    id BIGINT PRIMARY KEY,
    channel_id BIGINT
);