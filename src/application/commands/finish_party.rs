use anyhow::Ok;

use crate::{
    application::{error::ApplicationError, ports::PartyRepository},
    domain::party::Transfer,
};

pub struct FinishPartyCommand {
    pub chat_id: i64,
}

pub struct FinishPartyHandler<R>
where
    R: PartyRepository,
{
    party_repository: R,
}

impl<R> FinishPartyHandler<R>
where
    R: PartyRepository,
{
    pub fn new(party_repository: R) -> Self {
        Self { party_repository }
    }

    pub async fn execute(&self, command: FinishPartyCommand) -> anyhow::Result<Vec<Transfer>> {
        let party = self
            .party_repository
            .find_by_chat_id(command.chat_id)
            .await?;

        let Some(mut party) = party else {
            return Err(ApplicationError::NoPartyFound)?;
        };

        party.lock()?;

        let transfers = party.evaluate();

        self.party_repository.update_party(&party).await?;

        Ok(transfers)
    }
}
