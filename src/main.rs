use ansi_term::{ANSIString, Color::Fixed, Color::RGB};
use std::num::ParseIntError;
use twitchchat::{
    connector::smol::Connector,
    messages::{Commands, Privmsg},
    runner::AsyncRunner,
    twitch::Capability,
    Status, UserConfig,
};

fn get_color(hex: &str) -> ansi_term::Color {
    let hex = if hex.starts_with('#') {
        &hex[1..]
    } else {
        hex
    };

    match (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16))
        .collect::<Result<Vec<u8>, ParseIntError>>()
    {
        Ok(v) => RGB(v[0], v[1], v[2]),
        Err(_) => Fixed(15),
    }
}

fn colored_nick<'a>(
    name: &'a str,
    color: Option<&str>,
) -> ANSIString<'a> {
    let hex = match color {
        Some(c) => c.to_string(),
        None => "unset".to_string(),
    };

    get_color(&hex).paint(name)
}

async fn connect(
    user_config: &UserConfig,
    channel: &String,
) -> anyhow::Result<AsyncRunner> {
    let connector = Connector::twitch()?;
    let mut runner =
        AsyncRunner::connect(connector, user_config)
            .await?;
    let _ = runner.join(channel).await?;

    Ok(runner)
}

async fn read_loop(
    mut runner: AsyncRunner,
) -> anyhow::Result<()> {
    loop {
        match runner.next_message().await? {
            Status::Message(msg) => {
                handle(msg).await;
            }
            Status::Quit => {
                break;
            }
            Status::Eof => {
                break;
            }
        }
    }
    Ok(())
}

async fn handle(msg: Commands<'_>) {
    match msg {
        Commands::Privmsg(msg) => {
            show_message(msg).await
        }
        _ => {}
    }
}

async fn show_message(msg: Privmsg<'_>) {
    match msg.name() {
        "funtoon" => return,
        "botfrobber" => return,
        "cynanbot" => return,
        _ => {}
    };
    let nick = match msg.display_name() {
        Some(n) => n,
        None => msg.name(),
    };

    println!(
        "{} {}",
        colored_nick(nick, msg.tags().get("color")),
        msg.data()
    );
}

fn main() -> anyhow::Result<()> {
    let channel = std::env::var("TWITCH_CHANNEL")?;
    let fut = async move {
        let config = UserConfig::builder()
            .anonymous()
            .capabilities(&[Capability::Tags])
            .build()?;
        let runner =
            connect(&config, &channel.to_string())
                .await?;
        read_loop(runner).await
    };

    smol::block_on(fut)
}
