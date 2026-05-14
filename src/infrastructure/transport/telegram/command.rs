use teloxide::utils::command::BotCommands;

#[derive(BotCommands, Clone)]
#[command(rename_rule = "lowercase", description = "The commands I support:")]
pub enum Command {
    #[command(description = "display this text.")]
    Help,
    #[command(description = "start a new party.")]
    Start,
    #[command(description = "add expense to the party.", parse_with = "split")]
    Paid { amount: String, description: String },
    #[command(description = "include a member to the party.")]
    Part(String),
    #[command(description = "finish the party and calculate splits.")]
    End,
}
