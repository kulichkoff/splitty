mod command;

use command::Command;
use teloxide::{prelude::*, utils::command::BotCommands};

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
            let result = handler
                .execute(commands::StartPartyCommand {
                    chat_id: msg.chat.id.0,
                })
                .await;
            if let Err(err) = result {
                eprintln!("failed to execute start party command: {:?}", err);
            } else {
                bot.send_message(msg.chat.id, "A new party started. Tell me about payments so I can split expenses between the members").await?;
            }
        }
        Command::Paid(args) => {
            println!("paid args {}", args);
        }
        Command::Part(args) => {
            println!("part args {}", args);
        }
        Command::End => todo!(),
    };

    Ok(())
}
