use ansi_term::{ANSIString, Color::Fixed};
use phf::phf_map;
use twitchchat::{
    connector::smol::Connector,
    messages::{Commands, Privmsg},
    runner::AsyncRunner,
    twitch::Capability,
    Status, UserConfig,
};

static COLORS: phf::Map<
    &'static str,
    ansi_term::Color,
> = phf_map! {
    // unsure if these actually map to
    // correct twitch chatter colors
    "#FF0000" => Fixed(1),
    "#0000FF" => Fixed(4),
    "#008000" => Fixed(2),
    "#B22222" => Fixed(1),
    "#FF7F50" => Fixed(9),
    "#ADFF2F" => Fixed(10),
    "#FF4500" => Fixed(3),
    "#2E8B57" => Fixed(2),
    "#DAA520" => Fixed(11),
    "#D2691E" => Fixed(9),
    "#5F9EA0" => Fixed(6),
    "#1E90FF" => Fixed(12),
    "#FF69B4" => Fixed(13),
    "#8A2BE2" => Fixed(5),
    "#00FF7F" => Fixed(10)
};

fn get_color(
    hex: &str,
) -> Option<ansi_term::Color> {
    COLORS.get(hex).cloned()
}

fn colored_nick<'a>(
    name: &'a str,
    color: Option<&str>,
) -> ANSIString<'a> {
    let hex = match color {
        Some(c) => c.to_string(),
        None => "unset".to_string(),
    };

    match get_color(&hex) {
        Some(c) => c.paint(name),
        None => Fixed(15).paint(name),
    }
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
    println!(
        "{} {}",
        colored_nick(
            msg.name(),
            msg.tags().get("color")
        ),
        msg.data()
    );
}

fn main() -> anyhow::Result<()> {
    let channel =
        std::env::var("TWITCH_CHANNEL")?;
    let fut = async move {
        let config = UserConfig::builder()
            .anonymous()
            .capabilities(&[
                Capability::Tags,
            ])
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
