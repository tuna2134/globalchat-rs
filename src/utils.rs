use sqlx::MySqlPool;
use tokio::sync::RwLock;
use twilight_cache_inmemory::InMemoryCache;
use twilight_gateway::Shard;
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::Webhook,
    id::{marker::ChannelMarker, Id},
};

pub struct AppState {
    pub http: HttpClient,
    pub cache: InMemoryCache,
    pub shard: RwLock<Shard>,
    pub pool: MySqlPool,
}

pub async fn get_webhook(
    state: &AppState,
    channel_id: Id<ChannelMarker>,
) -> anyhow::Result<Webhook> {
    let webhooks = state
        .http
        .channel_webhooks(channel_id)
        .await?
        .model()
        .await?;
    let webhook = webhooks
        .iter()
        .find(|webhook| webhook.name == Some("globalchat-rs".to_string()));
    let webhook = if let Some(existed_webhook) = webhook {
        existed_webhook.clone()
    } else {
        state
            .http
            .create_webhook(channel_id, "globalchat-rs")?
            .await?
            .model()
            .await?
            .clone()
    };
    Ok(webhook)
}
