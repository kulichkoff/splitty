use crate::{application::ports::PartyRepository, domain::party::Party};

pub struct StartPartyCommand {
    pub chat_id: i64,
}

pub struct StartPartyHandler<R>
where
    R: PartyRepository,
{
    party_repository: R,
}

impl<R> StartPartyHandler<R>
where
    R: PartyRepository,
{
    pub fn new(party_repository: R) -> Self {
        Self { party_repository }
    }

    pub async fn execute(&self, command: StartPartyCommand) -> anyhow::Result<()> {
        let party = Party::new(command.chat_id);
        self.party_repository.save_party(party).await?;
        Ok(())
    }
}
