mod command;

use command::Command;
use teloxide::{prelude::*, utils::command::BotCommands};

pub async fn serve() {
    let bot = Bot::from_env();
    Command::repl(bot, handle_command).await;
}

async fn handle_command(bot: Bot, msg: Message, cmd: Command) -> ResponseResult<()> {
    match cmd {
        Command::Help => {
            bot.send_message(msg.chat.id, Command::descriptions().to_string())
                .await?
        }
        Command::Start => todo!(),
        Command::Paid {
            amount,
            description,
        } => {
            println!("{} {}", amount, description);
            todo!()
        }
        Command::Part(_) => todo!(),
        Command::End => todo!(),
    };

    Ok(())
}
