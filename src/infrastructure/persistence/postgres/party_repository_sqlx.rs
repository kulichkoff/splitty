use anyhow::Ok;
use sqlx::PgPool;

use crate::{
    application::ports::PartyRepository,
    domain::party::{MemberExpense, Party, PartyMember},
};

#[derive(Clone)]
pub struct SqlxPartyRepository {
    pool: PgPool,
}

impl SqlxPartyRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

impl PartyRepository for SqlxPartyRepository {
    async fn save_party(&self, party: &Party) -> anyhow::Result<i64> {
        let inserted = sqlx::query!(
            r#"
            INSERT INTO parties (
                chat_id,
                state
            ) VALUES ($1, $2)
            RETURNING id
            "#,
            party.chat_id(),
            party.state_str(),
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(inserted.id)
    }

    async fn update_party(&self, party: &Party) -> anyhow::Result<i64> {
        sqlx::query!(
            r#"
            UPDATE parties
            SET chat_id = $2,
                state = $3,
                updated_at = NOW()
            WHERE id = $1
            "#,
            party.id(),
            party.chat_id(),
            party.state_str()
        )
        .execute(&self.pool)
        .await?;

        Ok(party.id())
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

    async fn insert_member(&self, party: &Party, member: &PartyMember) -> anyhow::Result<i64> {
        let mut tx = self.pool.begin().await?;
        let insert_result = sqlx::query!(
            r#"
            INSERT INTO members (telegram_id, slug)
            VALUES ($1, $2)

            ON CONFLICT (telegram_id)
            DO UPDATE
            SET slug = members.slug
            RETURNING id
            "#,
            member.telegram_id(),
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
            party.id(),
            insert_result.id
        )
        .execute(&mut *tx)
        .await?;

        tx.commit().await?;
        Ok(insert_result.id)
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
