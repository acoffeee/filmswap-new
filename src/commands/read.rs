use crate::{
    Context, Error,
    database::{self, get_giftee_letter},
    types::Phase,
    utilities::{embed_builder, ensure_dm, ensure_has_giftee, ensure_joined},
};
use poise::CreateReply;
use rusqlite::Result;

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn read(ctx: Context<'_>) -> Result<(), Error> {
    if !ensure_joined(&ctx).await? {
        return Ok(());
    }
    if !ensure_dm(&ctx).await? {
        return Ok(());
    }
    if !ensure_has_giftee(&ctx).await? {
        return Ok(());
    }
    if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {
        return Ok(());
    }

    let letter = get_giftee_letter(ctx.author().id.get()).await;
    match letter {
        Ok(potential_letter) => match potential_letter {
            Some(l) => {
                let giftee_name: String;
                match database::get_giftee_name(ctx.author().id.get()).await {
                    Ok(name) => giftee_name = name,
                    Err(_) => giftee_name = "giftee".to_string()
                }
                let embed = embed_builder(
                    &l,
                    "Your giftee's letter",
                    "Dear Santa",
                    &format!("Love, {}", giftee_name),
                );
                let reply = CreateReply::default().embed(embed);
                ctx.send(reply).await?;
            }
            None => {
                ctx.say("Your giftee does not seem to have a letter")
                    .await?;
            }
        },
        Err(e) => {
            ctx.say("Error getting letter").await?;
            eprintln!("Error getting letter: {}", e);
        }
    }

    Ok(())
}
