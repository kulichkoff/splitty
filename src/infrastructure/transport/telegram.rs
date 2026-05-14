mod command;

use command::Command;
use teloxide::{prelude::*, sugar::request::RequestReplyExt, utils::command::BotCommands};

use crate::{AppContext, application::commands};

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
                eprintln!("failed to execute insert member command: {:?}", err);
            } else {
                bot.send_message(msg.chat.id, "✅").reply_to(msg.id).await?;
            }
        }
        Command::Part(args) => {
            println!("part args {}", args);
        }
        Command::End => todo!(),
    };

    Ok(())
}
