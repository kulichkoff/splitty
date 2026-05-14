use crate::domain::party::{MemberExpense, Party, PartyMember};

#[allow(async_fn_in_trait)]
pub trait PartyRepository: Send + Sync {
    async fn save_party(&self, party: Party) -> anyhow::Result<()>;
    async fn insert_member(&self, party: &Party, member: &PartyMember) -> anyhow::Result<()>;
    async fn insert_member_expense(
        &self,
        party: &Party,
        member: &PartyMember,
        expense: &MemberExpense,
    ) -> anyhow::Result<()>;
    async fn find_by_chat_id(&self, chat_id: i64) -> anyhow::Result<Option<Party>>;
}
