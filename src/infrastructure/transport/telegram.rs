mod command;

use command::Command;
use teloxide::{prelude::*, sugar::request::RequestReplyExt, utils::command::BotCommands};

use crate::{AppContext, application::commands, domain::party::Transfer};

pub async fn serve(ctx: AppContext) {
    let bot = Bot::from_env();

    Command::repl(bot, move |bot, msg, cmd| {
        let ctx = ctx.clone();

        async move { handle_command(bot, msg, cmd, ctx).await }
    })
    .await;
}

async fn handle_command(
    bot: Bot,
    msg: Message,
    cmd: Command,
    ctx: AppContext,
) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?;
        }
        Command::Start => {
            let party_repo = ctx.party_repo;
            let handler = commands::StartPartyHandler::new(party_repo.as_ref().clone());
            let owner = msg.from.unwrap();
            let result = handler
                .execute(commands::StartPartyCommand {
                    chat_id: msg.chat.id.0,
                    owner_slug: owner.username.unwrap(),
                    owner_telegram_id: owner.id.0 as i64,
                })
                .await;
            if let Err(err) = result {
                eprintln!("failed to execute start party command: {:?}", err);
            } else {
                bot.send_message(msg.chat.id, "A new party started. Tell me about payments so I can split expenses between the members").await?;
            }
        }
        Command::Paid(args) => {
            const USAGE: &'static str = "Usage: /paid <amount> <description>";

            let mut parts = args.splitn(2, ' ');

            let Some(amount) = parts.next() else {
                bot.send_message(msg.chat.id, USAGE).await?;
                return Ok(());
            };

            let Ok(amount_parsed) = amount.parse::<f64>() else {
                bot.send_message(msg.chat.id, USAGE).await?;
                return Ok(());
            };
            let amount_cents = (amount_parsed * 100f64) as i64;

            let description = parts.next().map(str::to_string);
            let sender = msg.from.unwrap();
            let sender_username = sender.username.unwrap();

            let party_repo = ctx.party_repo;
            let handler = commands::AddExpenseHandler::new(party_repo.as_ref().clone());

            let result = handler
                .execute(commands::AddExpenseCommand {
                    chat_id: msg.chat.id.0,
                    member_telegram_id: sender.id.0 as i64,
                    member_slug: sender_username.clone(),
                    amount_cents,
                    description,
                })
                .await;

            if let Err(err) = result {
                eprintln!("failed to execute add expense command: {:?}", err);
            } else {
                bot.send_message(msg.chat.id, "✅").reply_to(msg.id).await?;
            }
        }
        Command::Part(args) => {
            let sender = msg.from.unwrap();
            let sender_username = sender.username.unwrap();

            let party_repo = ctx.party_repo;
            let handler = commands::IncludeMemberHandler::new(party_repo.as_ref().clone());

            let result = handler
                .execute(commands::IncludeMemberCommand {
                    chat_id: msg.chat.id.0,
                    member_telegram_id: sender.id.0 as i64,
                    member_slug: sender_username,
                })
                .await;

            if let Err(err) = result {
                eprintln!("failed to execute include member command: {:?}", err);
            } else {
                bot.send_message(msg.chat.id, "You're listed now")
                    .reply_to(msg.id)
                    .await?;
            }
        }
        Command::End => {
            let party_repo = ctx.party_repo;
            let handler = commands::FinishPartyHandler::new(party_repo.as_ref().clone());

            let result = handler
                .execute(commands::FinishPartyCommand {
                    chat_id: msg.chat.id.0,
                })
                .await;

            match result {
                Err(err) => {
                    eprintln!("failed to execute finish party command: {:?}", err);
                }
                Ok(transfers) => {
                    let msg_text = build_transfers_message(transfers);
                    bot.send_message(msg.chat.id, msg_text).await?;
                }
            }
        }
    };

    Ok(())
}

fn build_transfers_message(transfers: Vec<Transfer>) -> String {
    let mut msg = String::from("Party is finished, let's do some money\n");

    if transfers.is_empty() {
        msg.push_str("It looks like you've already balanced all the payments.");
        return msg;
    }

    for transfer in transfers {
        msg.push_str(&format!(
            "\n{} sends {} to {}",
            transfer.from_id, transfer.amount, transfer.to_id
        ));
    }

    msg
}
