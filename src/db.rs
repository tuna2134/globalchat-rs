use sqlx::{pool, MySqlPool};

pub async fn create_message(
    pool: &MySqlPool,
    message_id: i64,
    channel_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO message (id, channel_id) VALUES (?, ?);",
        message_id,
        channel_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

