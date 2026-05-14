use crate::application::{error::ApplicationError, ports::PartyRepository};
use crate::domain::party::PartyMember;

pub struct IncludeMemberCommand {
    pub chat_id: i64,
    pub member_telegram_id: i64,
    pub member_slug: String,
}

pub struct IncludeMemberHandler<R>
where
    R: PartyRepository,
{
    party_repository: R,
}

impl<R> IncludeMemberHandler<R>
where
    R: PartyRepository,
{
    pub fn new(party_repository: R) -> Self {
        Self { party_repository }
    }

    pub async fn execute(&self, command: IncludeMemberCommand) -> anyhow::Result<i64> {
        let party = self
            .party_repository
            .find_by_chat_id(command.chat_id)
            .await?;

        let Some(party) = party else {
            return Err(ApplicationError::NoPartyFound)?;
        };

        let member = &PartyMember::new(command.member_telegram_id, &command.member_slug);
        let member_id = self.party_repository.insert_member(&party, member).await?;
        Ok(member_id)
    }
}
