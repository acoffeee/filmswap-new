use crate::database;
use crate::types::Phase::{self, Join, Swap, Watch};
use crate::{Context, Error, utilities::{ensure_host_role, embed_builder}};
use poise::CreateReply;
use serenity::all::{CreateMessage, UserId};
#[poise::command(prefix_command, track_edits, slash_command)]
pub async fn set_phase(
    ctx: Context<'_>,
    #[description = "The phase you want to set it to"] phase: Phase,
) -> Result<(), Error> {
    if !ensure_host_role(&ctx, ctx.author()).await? {
        return Ok(());
    };

    let current_phase = database::get_phase()?;
    if current_phase == phase {
        let message = format!("You are already in this phase!");
        ctx.send(CreateReply::default().content(message).ephemeral(true))
            .await?;
        return Ok(());
    }

    match phase {
        Phase::Join => {
            if current_phase == Swap {
                let message = format!("You cannot change from the swap to the join phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                let res = database::reset_giftees_and_submissions();
                match res {
                    Ok(_) => {
                        database::set_phase(phase)?;
                        let message = format!("Succesfully changed phase from watch to join.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                    }
                    Err(e) => {
                        let message =
                            format!("Error changing from watch to join and resetting gifts.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                        eprintln!("Error changing phases and ressetting gifts: {}", e);
                    }
                }
            }
        }
        Phase::Swap => {
            if current_phase == Watch {
                let message = format!("You cannot change from the watch to the swap phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                let res = database::match_users();
                match res {
                    Ok(_) => {
                        database::set_phase(phase)?;
                        let message = format!("Succesfully changed phase from watch to swap.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                        if let Err(e) = send_out_letters(&ctx).await  {
                            eprintln!("Error sending out letters: {:?}", e);
                        }
                    }
                    Err(e) => {
                        let message =
                            format!("Error changing from watch to join and matching users.");
                        ctx.send(CreateReply::default().content(message).ephemeral(true))
                            .await?;
                        eprintln!("Error changing phases and matching users: {}", e);
                    }
                }
            }
        }
        Phase::Watch => {
            if current_phase == Join {
                let message = format!("You cannot change from the join to the watch phase.");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            } else {
                database::set_phase(phase)?;
                let message = format!("Changed phase to watch");
                ctx.send(CreateReply::default().content(message).ephemeral(true))
                    .await?;
            }
        }
    }
    Ok(())
}
async fn send_out_letters(ctx: &Context<'_>) -> Result<(), Error>{
    let user_ids = match database::get_matching_order() {
        Ok(users) => {
            let mut uids: Vec<u64> = Vec::new();
            users.into_iter()
                 .for_each(
                    |user| uids.push(user.0)
                 );
            uids
        }
        Err(e) => {
            ctx.send(
                CreateReply::default()
                .content("Failed to get the matching orders of giftees, but succefully swapped periods")
                .ephemeral(true)
            )
            .await?;
            return Ok(());
        }
    };
    for santa_id in user_ids {
        let santa = UserId::new(santa_id);
        let giftee_letter = database::get_giftee_letter(santa_id).await;
        match giftee_letter {
            Ok(potential_letter) => match potential_letter {
                Some(l) => {
                    let giftee_name: String;
                    match database::get_giftee_name(santa_id).await {
                        Ok(name) => giftee_name = name,
                        Err(_) => giftee_name = "giftee".to_string()
                    }
                    let embed = embed_builder(
                        &l,
                        "Your giftee's letter",
                        "Dear Santa",
                        &format!("Love, {}", giftee_name),
                    );
                    let message = CreateMessage::new().embed(embed);
                    santa.dm(&ctx.http(), message).await?;
                }
                None => {
                    let giftee_name: String;
                    match database::get_giftee_name(santa_id).await {
                        Ok(name) => giftee_name = name,
                        Err(_) => giftee_name = "giftee".to_string()
                    }
                    let message = CreateMessage::default().content(format!("Your giftee ({}) does not have a letter", giftee_name));
                    santa.dm(&ctx.http(),message).await?;
                }
            },
            Err(e) => {
                eprintln!("Error getting letter: {}", e);
                return Ok(());
            }
        }
    }
    return Ok(());
}