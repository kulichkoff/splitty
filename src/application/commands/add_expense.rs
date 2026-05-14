use crate::{
    application::{
        commands::{IncludeMemberCommand, IncludeMemberHandler},
        error::ApplicationError,
        ports::PartyRepository,
    },
    domain::party::{MemberExpense, PartyMember},
};

pub struct AddExpenseCommand {
    pub chat_id: i64,
    pub member_telegram_id: i64,
    pub member_slug: String,
    pub amount_cents: i64,
    pub description: Option<String>,
}

pub struct AddExpenseHandler<R>
where
    R: PartyRepository,
{
    party_repository: R,
}

impl<R> AddExpenseHandler<R>
where
    R: PartyRepository,
{
    pub fn new(party_repository: R) -> Self {
        Self { party_repository }
    }

    pub async fn execute(&self, command: AddExpenseCommand) -> anyhow::Result<()> {
        let party = self
            .party_repository
            .find_by_chat_id(command.chat_id)
            .await?;

        let Some(party) = party else {
            return Err(ApplicationError::NoPartyFound)?;
        };

        let member = party.get_member(command.member_slug.clone());
        let member: &PartyMember = match member {
            Some(member) => member,
            None => {
                let include_member_handler: IncludeMemberHandler<R> =
                    IncludeMemberHandler::new(self.party_repository.clone());

                let member_id = include_member_handler
                    .execute(IncludeMemberCommand {
                        chat_id: command.chat_id,
                        member_telegram_id: command.member_telegram_id,
                        member_slug: command.member_slug.clone(),
                    })
                    .await?;

                &PartyMember::with_id(member_id, command.member_telegram_id, &command.member_slug)
            }
        };

        let expense = &MemberExpense::new(command.amount_cents, command.description);
        self.party_repository
            .insert_member_expense(&party, member, expense)
            .await?;
        Ok(())
    }
}
