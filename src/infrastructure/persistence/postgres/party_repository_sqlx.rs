use anyhow::Ok;
use sqlx::PgPool;

use crate::{
    application::ports::PartyRepository,
    domain::party::{MemberExpense, Party, PartyMember},
};

pub struct SqlxPartyRepository {
    pool: PgPool,
}

impl SqlxPartyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PartyRepository for SqlxPartyRepository {
    async fn save_party(&self, party: Party) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO parties (
                chat_id,
                state
            ) VALUES ($1, $2)
            "#,
            party.chat_id(),
            party.state_str(),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn find_by_chat_id(&self, chat_id: i64) -> anyhow::Result<Option<Party>> {
        let party_row = sqlx::query!(
            r#"
            SELECT id, state, chat_id
            FROM parties
            WHERE chat_id = $1
            ORDER BY created_at DESC
            LIMIT 1
            "#,
            chat_id
        )
        .fetch_optional(&self.pool)
        .await?;

        let Some(row) = party_row else {
            return Ok(None);
        };

        let party_expenses_rows = sqlx::query!(
            r#"
            SELECT amount_cents, member_id, description
            FROM expenses
            WHERE party_id = $1
            "#,
            row.id
        )
        .fetch_all(&self.pool)
        .await?;

        let members: Vec<PartyMember> = sqlx::query!(
            r#"
            SELECT m.id, m.slug, m.telegram_id FROM members m
            JOIN party_members pm ON pm.member_id = m.id
            WHERE pm.party_id = $1
            "#,
            row.id
        )
        .fetch_all(&self.pool)
        .await?
        .into_iter()
        .map(|r| {
            let member_expenses: Vec<MemberExpense> = party_expenses_rows
                .iter()
                .filter(|exp| exp.member_id == r.id)
                .map(|exp| MemberExpense::new(exp.amount_cents, exp.description.clone()))
                .collect();
            PartyMember::with_expenses(r.id, r.telegram_id, &r.slug, member_expenses)
        })
        .collect();

        Ok(Some(Party::from_raw(
            row.id,
            row.chat_id,
            members,
            &row.state,
        )))
    }

    async fn insert_member(&self, party: &Party, member: &PartyMember) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO members (telegram_id, slug)
            VALUES ($1, $2)

            ON CONFLICT (telegram_id)
            DO NOTHING

            RETURNING id
            "#,
            0, // TODO: pass real tg id
            member.slug()
        )
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query!(
            r#"
            INSERT INTO party_members (party_id, member_id)
            VALUES ($1, $2)

            ON CONFLICT DO NOTHING
            "#,
            party.id(), // TODO: pass real tg id
            insert_result.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(())
    }

    async fn insert_member_expense(
        &self,
        party: &Party,
        member: &PartyMember,
        expense: &MemberExpense,
    ) -> anyhow::Result<()> {
        sqlx::query!(
            r#"
            INSERT INTO expenses (
                amount_cents,
                party_id,
                member_id,
                description
            ) VALUES ($1, $2, $3, $4)
            "#,
            expense.amount_cents(),
            party.id(),
            member.id(),
            expense.description()
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }
}
