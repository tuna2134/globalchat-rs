use sqlx::MySqlPool;

pub async fn create_message(
    pool: &MySqlPool,
    original_message_id: i64,
    message_id: i64,
    channel_id: i64,
) -> anyhow::Result<()> {
    sqlx::query!(
        "INSERT INTO message (original_message_id, id, channel_id) VALUES (?, ?, ?);",
        original_message_id,
        message_id,
        channel_id
    )
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn get_messages(
    pool: &MySqlPool,
    original_message_id: i64,
) -> anyhow::Result<Vec<(i64, i64)>> {
    let messages = sqlx::query!(
        "SELECT id, channel_id FROM message WHERE original_message_id = ?;",
        original_message_id
    )
    .fetch_all(pool)
    .await?;
    Ok(messages.into_iter().map(|m| (m.id, m.channel_id)).collect())
}
