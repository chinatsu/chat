use ansi_term::{
    ANSIString, Color::Fixed, Color::RGB,
};
use base64::{
    engine::general_purpose, Engine as _,
};
use itertools::Itertools;
use std::num::ParseIntError;
use twitchchat::{
    connector::smol::Connector,
    messages::{Commands, Privmsg},
    runner::AsyncRunner,
    twitch::Capability,
    Status, UserConfig,
};

type Hex = Result<Vec<u8>, ParseIntError>;

fn get_color(hex: &str) -> ansi_term::Color {
    let hex = if hex.starts_with('#') {
        &hex[1..]
    } else {
        hex
    };

    if hex.len() != 6 {
        return RGB(100, 100, 100);
    }

    match (0..hex.len())
        .step_by(2)
        .map(|i| {
            u8::from_str_radix(&hex[i..i + 2], 16)
        })
        .collect::<Hex>()
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
    let mut runner = AsyncRunner::connect(
        connector,
        user_config,
    )
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
                handle(msg).await?;
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

async fn handle(
    msg: Commands<'_>,
) -> anyhow::Result<()> {
    match msg {
        Commands::Privmsg(msg) => {
            Ok(show_message(msg).await?)
        }
        _ => Ok(()),
    }
}

async fn emote_to_image(
    emote_id: &str,
) -> anyhow::Result<Vec<u8>> {
    let bytes = reqwest::blocking::get(
        format!(
            "https://static-cdn.jtvnw.net/emoticons/v2/{}/static/light/1.0", 
            emote_id)
        )?
        .bytes()?;
    let encoded =
        general_purpose::STANDARD.encode(&bytes);
    if encoded.len() > 4096 {
        // ignore the emote if it's too large
        return Ok(vec![]);
    }
    Ok(format!(
        "\x1b_Ga=T,f=100,r=1,m=0;{encoded}\x1b\\",
    )
    .into_bytes())
}

async fn populate_emotes(
    msg: &Privmsg<'_>,
) -> anyhow::Result<String> {
    let mut message =
        String::from(msg.data()).into_bytes();

    for emote in msg.iter_emotes() {
        let img =
            emote_to_image(&emote.id).await?;
        for range in emote
            .ranges
            .iter()
            .sorted_by_key(|r| r.start)
            .rev()
        {
            message.splice(
                range.start as usize
                    ..range.end as usize + 1,
                img.iter().cloned(),
            );
        }
    }

    Ok(String::from_utf8_lossy(&message)
        .into_owned())
}

async fn show_message(
    msg: Privmsg<'_>,
) -> anyhow::Result<()> {
    match msg.name() {
        "funtoon" => return Ok(()),
        "botfrobber" => return Ok(()),
        "cynanbot" => return Ok(()),
        "nightbot" => return Ok(()),
        "streamelements" => return Ok(()),
        _ => {}
    };
    let nick = match msg.display_name() {
        Some(n) => n,
        None => msg.name(),
    };

    let message = populate_emotes(&msg)
        .await?
        .trim()
        .to_string();
    if message.len() == 0 {
        return Ok(());
    }
    println!(
        "{} {}",
        colored_nick(
            nick,
            msg.tags().get("color")
        ),
        message
    );

    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args: Vec<String> =
        std::env::args().collect();

    if args.len() < 2 {
        println!(
            "Specify desired Twitch channel"
        );
        return Ok(());
    }
    let channel = &args[1];
    let fut = async move {
        let config = UserConfig::builder()
            .anonymous()
            .capabilities(&[Capability::Tags])
            .build()?;
        let runner = connect(
            &config,
            &channel.to_string(),
        )
        .await?;
        read_loop(runner).await
    };

    smol::block_on(fut)
}
