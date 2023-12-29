use std::sync::Arc;

use crate::{
    event_handler::{HttpKey, TrackErrorNotifier},
    helpers::get_episode_link,
};
use serenity::{
    client::Context,
    framework::standard::{
        macros::{command, group},
        Args, CommandResult,
    },
    model::channel::Message,
    Result as SerenityResult,
};
use songbird::input::YoutubeDl;
use songbird::{events::TrackEvent, Songbird};

#[group]
#[commands(help, join, leave, mute, play, unmute)]
pub struct General;

#[command]
async fn help(ctx: &Context, msg: &Message) -> CommandResult {
    msg_reply(msg, ctx, "

        A bot that plays some tunes to keep you from getting bored!

        Commands:

        !join: Joins the voice channel of the caller. \
        Don't forget to join one before doing so! ;)

        !play {Track #} [loop]: Starts playing the specified episode. The second argument can be \"loop\" if you wish to loop the track.
        So to play the seventh track and loop it, you'd say:
            !play 7 loop

        !mute: Mutes the bot.

        !unmute: Unmutes the bot.

        !leave: Leaves the voice channel.
",
        )
        .await;

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn join(ctx: &Context, msg: &Message) -> CommandResult {
    let (guild_id, channel_id) = {
        let guild = msg.guild(&ctx.cache).unwrap();
        let channel_id = guild
            .voice_states
            .get(&msg.author.id)
            .and_then(|voice_state| voice_state.channel_id);

        (guild.id, channel_id)
    };

    let connect_to = match channel_id {
        Some(channel) => channel,
        None => {
            msg_reply(msg, ctx, "Try again after joining a voice channel! ;)").await;

            return Ok(());
        }
    };

    let manager = get_songbird(ctx).await;

    if let Ok(handler_lock) = manager.join(guild_id, connect_to).await {
        msg_reply(msg, ctx, "Joined your channel!").await;
        // Attach an event handler to see notifications of all track errors.
        let mut handler = handler_lock.lock().await;
        handler.add_global_event(TrackEvent::Error.into(), TrackErrorNotifier);
        if let Err(e) = handler.deafen(true).await {
            msg_say(msg, ctx, format!("Failed to deafen: {e}")).await;
        }
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn play(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let episode_num = match args.single::<String>() {
        Ok(episode_num) => episode_num,
        Err(_) => {
            msg_say(
                msg,
                ctx,
                "Enter the number of the episode that you wish to play",
            )
            .await;

            return Ok(());
        }
    };

    let enable_loop = match args.single::<String>() {
        Ok(second_argument) => {
            if second_argument.to_lowercase() == "loop" {
                true
            } else {
                msg_reply(
                    msg,
                    ctx,
                    "The second argument should either be \"loop\" or empty",
                )
                .await;
                return Ok(());
            }
        }
        Err(_) => false,
    };
    let episode_num = match episode_num.parse::<i32>() {
        Ok(num) => num,
        Err(_) => {
            msg_reply(msg, ctx, "Please enter the number of the episode in digits").await;
            return Ok(());
        }
    };

    if !(1..=70).contains(&episode_num) {
        msg_reply(msg, ctx, "Please enter a number between 1..70").await;
        return Ok(());
    }

    let guild_id = msg.guild_id.unwrap();

    let http_client = {
        let data = ctx.data.read().await;
        data.get::<HttpKey>()
            .cloned()
            .expect("Guaranteed to exist in the typemap.")
    };

    let manager = get_songbird(ctx).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;

        let src = YoutubeDl::new(http_client, get_episode_link(episode_num));
        let _episode_handle = handler.play_only_input(src.clone().into());
        if enable_loop {
            match _episode_handle.enable_loop() {
                Ok(_) => msg_reply(msg, ctx, "Loop enabled!").await,
                Err(e) => msg_reply(msg, ctx, format!("Failed to enable loop: {e}")).await,
            }
        }

        msg_say(msg, ctx, format!("Now playing: Episode {episode_num}")).await;
    } else {
        msg_say(msg, ctx, "Not in a voice channel to play in :c").await;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn mute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = get_songbird(ctx).await;

    let handler_lock = match manager.get(guild_id) {
        Some(handler) => handler,
        None => {
            msg_reply(msg, ctx, "Not in a voice channel").await;

            return Ok(());
        }
    };

    let mut handler = handler_lock.lock().await;

    if handler.is_mute() {
        msg_say(msg, ctx, "Already muted").await;
    } else {
        if let Err(e) = handler.mute(true).await {
            msg_say(msg, ctx, format!("Failed to mute: {e}")).await;
        }

        msg_say(msg, ctx, "Now muted").await;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn unmute(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = get_songbird(ctx).await;

    if let Some(handler_lock) = manager.get(guild_id) {
        let mut handler = handler_lock.lock().await;
        if let Err(e) = handler.mute(false).await {
            msg_say(msg, ctx, format!("Failed to unmute: {e}")).await;
        }

        msg_say(msg, ctx, "Now unmuted!").await;
    } else {
        msg_say(msg, ctx, "Not in a voice channel to be unmuted in").await;
    }

    Ok(())
}

#[command]
#[only_in(guilds)]
async fn leave(ctx: &Context, msg: &Message) -> CommandResult {
    let guild_id = msg.guild_id.unwrap();

    let manager = get_songbird(ctx).await;
    let has_handler = manager.get(guild_id).is_some();

    if has_handler {
        if let Err(e) = manager.remove(guild_id).await {
            msg_say(msg, ctx, format!("Failed to leave: {e}")).await
        }
        msg_say(msg, ctx, "Bye bye!").await;
    } else {
        msg_reply(msg, ctx, "Not in a voice channel").await;
    }

    Ok(())
}

/// Checks that a message successfully sent; if not, then logs why to stdout.
fn check_msg(result: SerenityResult<Message>) {
    if let Err(why) = result {
        println!("Error sending message: {:?}", why);
    }
}

async fn msg_say(sent_msg: &Message, ctx: &Context, content: impl Into<String>) {
    check_msg(sent_msg.channel_id.say(ctx, content).await);
}

async fn msg_reply(sent_msg: &Message, ctx: &Context, content: impl Into<String>) {
    check_msg(sent_msg.reply(&ctx.http, content).await);
}

async fn get_songbird(ctx: &Context) -> Arc<Songbird> {
    songbird::get(ctx)
        .await
        .expect("Songbird Voice client placed in at initialisation.")
        .clone()
}
