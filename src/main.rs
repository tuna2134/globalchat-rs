use std::{env, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;

struct AppState {
    http: HttpClient,
    cache: InMemoryCache,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let token: String = env::var("DISCORD_TOKEN")?;

    let intents: Intents = Intents::GUILD_MESSAGES | Intents::GUILDS;
    let mut shard: Shard = Shard::new(ShardId::ONE, token.clone(), intents);
    let http: HttpClient = HttpClient::new(token.clone());
    let cache: InMemoryCache = InMemoryCache::builder()
        .resource_types(ResourceType::CHANNEL)
        .build();
    let state: Arc<AppState> = Arc::new(AppState { http, cache: cache });
    loop {
        let event: Event = match shard.next_event().await {
            Ok(event) => event,
            Err(error) => {
                log::error!("Error receiving event: {:?}", error);
                if error.is_fatal() {
                    break;
                }
                continue;
            }
        };
        state.cache.update(&event);
        tokio::spawn(handle_event(Arc::clone(&state), event));
    }
    Ok(())
}

async fn handle_event(state: Arc<AppState>, event: Event) -> anyhow::Result<()> {
    match event {
        Event::Ready(_) => {
            log::info!("The bot is ready!");
        }
        Event::MessageCreate(message) => {
            let channel = state.cache.channel(message.channel_id);
            if let Some(channel) = channel {
                if channel.name != Some("globalchat-rs".to_string()) {
                    return Ok(());
                }
            }
            for channel in state.cache.iter().channels() {
                if channel.name != Some("globalchat-rs".to_string()) {
                    continue;
                }
                let webhooks = state
                    .http
                    .channel_webhooks(channel.id)
                    .await?
                    .model()
                    .await?;
                let webhook = webhooks
                    .iter()
                    .find(|webhook| webhook.name == Some("globalchat-rs".to_string()));
                let webhook = if let Some(existed_webhook) = webhook {
                    existed_webhook
                } else {
                    &state
                        .http
                        .create_webhook(channel.id, "globalchat-rs")?
                        .await?
                        .model()
                        .await?
                };
            }
        }
        _ => {}
    }
    Ok(())
}
