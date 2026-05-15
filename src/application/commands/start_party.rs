use crate::{
    application::ports::PartyRepository,
    domain::party::{Party, PartyMember},
};

pub struct StartPartyCommand {
    pub chat_id: i64,
    pub owner_telegram_id: i64,
    pub owner_slug: String,
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
        let mut party = Party::new(command.chat_id);
        let party_id = self.party_repository.save_party(&party).await?;
        party.set_id(party_id);

        let member = PartyMember::new(command.owner_telegram_id, &command.owner_slug);
        self.party_repository.insert_member(&party, &member).await?;

        Ok(())
    }
}
