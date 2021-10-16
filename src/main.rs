use anyhow::{Context as _, Result};
use once_cell::sync::Lazy;
use regex::Regex;
use serenity::{client::ClientBuilder, model::channel::Message, prelude::*};

fn env_var(name: &str) -> Result<String> {
    std::env::var(name).with_context(|| format!("failed to get {} environment variable", name))
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv::dotenv().ok();

    let use_ansi = env_var("NO_COLOR").is_err();

    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .with_ansi(use_ansi)
        .init();

    ClientBuilder::new(&env_var("DISCORD_TOKEN")?)
        .event_handler(EvHandler)
        .await?
        .start()
        .await?;

    Ok(())
}

struct EvHandler;

#[serenity::async_trait]
impl EventHandler for EvHandler {
    async fn message(&self, ctx: Context, message: Message) {
        if message.author.bot {
            return;
        }

        static COMMAND_REGEX: Lazy<Regex> = Lazy::new(|| {
            Regex::new(r#"^`?(?:tokio::|(?:std::)?thread::)spawn(?:\((?:"(?P<name>.+)")?\))?`?$"#)
                .unwrap()
        });

        let captures = COMMAND_REGEX.captures(&message.content);

        if let Some(captures) = captures {
            let name = captures
                .name("name")
                .map(|x| x.as_str())
                .unwrap_or_else(|| {
                    if message.content.contains("tokio") {
                        "tokio-runtime-worker"
                    } else {
                        "unnamed"
                    }
                });

            message
                .channel_id
                .create_public_thread(&ctx.http, message.id, |t| t.name(name))
                .await
                .context("failed to create thread")
                .unwrap();
        }
    }
}
