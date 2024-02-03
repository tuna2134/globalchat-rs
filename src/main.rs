use std::{env, sync::Arc};
use tokio::sync::RwLock;
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Event, Intents, Shard, ShardId};
use twilight_http::Client as HttpClient;
use twilight_model::{
    channel::message::AllowedMentions, gateway::{
        payload::outgoing::UpdatePresence,
        presence::{Activity, ActivityType, Status},
    }, http::attachment::Attachment,
};
use sqlx::MySqlPool;
mod db;
mod utils;
use utils::AppState;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    log::info!("Now booting...");
    let token: String = env::var("DISCORD_TOKEN")?;

    let intents: Intents = Intents::GUILD_MESSAGES | Intents::GUILDS | Intents::MESSAGE_CONTENT;
    let shard: RwLock<Shard> = RwLock::new(Shard::new(ShardId::ONE, token.clone(), intents));
    let http: HttpClient = HttpClient::new(token.clone());
    let cache: InMemoryCache = InMemoryCache::builder()
        .resource_types(ResourceType::CHANNEL)
        .build();
    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;
    sqlx::migrate!().run(&pool).await?; 
    let state: Arc<AppState> = Arc::new(AppState { http, cache, shard, pool });
    loop {
        let event: Event = match state.shard.write().await.next_event().await {
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
            state
                .shard
                .write()
                .await
                .command(&UpdatePresence::new(
                    vec![Activity {
                        application_id: None,
                        assets: None,
                        buttons: Vec::new(),
                        created_at: None,
                        details: None,
                        emoji: None,
                        flags: None,
                        id: None,
                        instance: None,
                        kind: ActivityType::Playing,
                        name: format!("v{}", env!("CARGO_PKG_VERSION")),
                        party: None,
                        secrets: None,
                        state: None,
                        timestamps: None,
                        url: None,
                    }],
                    true,
                    None,
                    Status::Online,
                )?)
                .await?;
            log::info!("The bot is ready!");
        }
        Event::MessageCreate(message) => {
            if message.author.bot {
                return Ok(());
            }
            let channel = state.cache.channel(message.channel_id);
            if let Some(channel) = channel {
                if channel.name != Some("globalchat-rs".to_string()) {
                    return Ok(());
                }
            }
            let mut attachments: Vec<Attachment> = Vec::new();
            for attachment in &message.attachments {
                let data = reqwest::Client::new()
                    .get(&attachment.url)
                    .send()
                    .await?
                    .bytes()
                    .await?;
                attachments.push(Attachment::from_bytes(
                    attachment.filename.clone(),
                    data.to_vec(),
                    attachment.id.get(),
                ));
            }
            for channel in state.cache.iter().channels() {
                if channel.name != Some("globalchat-rs".to_string()) {
                    continue;
                }
                if channel.id == message.channel_id {
                    continue;
                }
                let webhook = utils::get_webhook(&state, channel.id).await?;
                let avatar_hash = if let Some(avatar) = message.author.avatar.as_ref() {
                    avatar.to_string()
                } else if message.author.discriminator == 0 {
                    (message.author.id.get() >> (22 % 6)).to_string()
                } else {
                    (message.author.discriminator % 5).to_string()
                };
                state
                    .http
                    .execute_webhook(webhook.id, &webhook.token.unwrap_or("".to_string()))
                    .content(&message.content)?
                    .attachments(&attachments)?
                    .username(&message.author.name)?
                    .avatar_url(&format!(
                        "https://cdn.discordapp.com/avatars/{}/{}.png",
                        message.author.id, avatar_hash
                    ))
                    .allowed_mentions(Some(&AllowedMentions::default()))
                    .wait()
                    .await?
                    .model()
                    .await?;
            }
        }
        _ => {}
    }
    Ok(())
}
