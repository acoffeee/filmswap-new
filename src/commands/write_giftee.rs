use crate::{
    Context, Error, database,
    types::Phase,
    utilities::{
        self, ensure_dm, ensure_embed_field_lenght, ensure_has_giftee, ensure_joined,
        reject_if_already_running, wait_for_message_with_cancel,
    },
};
use rusqlite::Result;
use serenity::all::{CreateMessage, UserId};

#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn write_giftee(ctx: Context<'_>) -> Result<(), Error> {
    reject_if_already_running(&ctx, || async {
        if !ensure_joined(&ctx).await? { return Ok(()); }
        if !ensure_dm(&ctx).await? { return Ok(()); }
        if !ensure_has_giftee(&ctx).await? { return Ok(()); }
        if !crate::utilities::ensure_correct_phase(&ctx, vec![Phase::Swap, Phase::Watch]).await? {return Ok(())}

        let prompt_message = "
        Press the cancel button to cancel the action, otherwise send a message to send something to your giftee (the person you will give something).
        ";
        let user_id = ctx.author().id.get();

        match wait_for_message_with_cancel(&ctx, prompt_message).await? {
            Some(message) => {
                if !ensure_embed_field_lenght(&ctx, &message, 2000).await? {
                    return Ok(());
                }

                match database::get_giftee(user_id).await {
                    Ok(giftee) => {
                        let giftee_id = UserId::new(giftee);
                        let user_name = match database::get_giftee_name(user_id).await {
                            Ok(name) => name,
                            Err(_) => "giftee".to_string()
                        };

                        let embed = utilities::embed_builder(
                            &message,
                            "Your Santa sent you a message",
                            &format!("Dear {}", user_name),
                            "Love, Santa",
                        );

                        let giftee_message = CreateMessage::new().embed(embed);

                        let _ = match giftee_id.dm(&ctx.http(), giftee_message).await {
                            Ok(_) => ctx.say("Message sent successfully to your giftee").await?,
                            Err(e) => {
                                eprintln!("Error sending message to giftee: {}", e);
                                ctx.say("An error occurred sending your message").await?
                            }
                        };
                    }
                    Err(e) => {
                        eprintln!("Error fetching giftee: {}", e);
                        ctx.say("An error occurred getting your giftee").await?;
                    }
                }
            }
            None => {()}
        }

        Ok(())
    }).await
}
