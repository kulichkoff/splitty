use std::collections::HashMap;

use thiserror::Error;

#[derive(Debug)]
pub struct MemberExpense {
    expense: f64,
    description: Option<String>,
}

impl MemberExpense {
    pub fn new(amount_cents: i64, description: Option<String>) -> Self {
        let amount = (amount_cents as f64) / 100f64;
        Self {
            expense: amount,
            description,
        }
    }

    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    pub fn amount_cents(&self) -> i64 {
        (self.expense * 100f64).trunc() as i64
    }

    pub fn expense(&self) -> f64 {
        self.expense
    }
}

type PartyMemberId = i64;
type PartyMemberSlug = String;

#[derive(Debug)]
pub struct PartyMember {
    id: PartyMemberId,
    telegram_id: i64,
    slug: PartyMemberSlug,
    to_transfer: f64,
    expenses: Vec<MemberExpense>,
}

impl PartyMember {
    pub fn new(telegram_id: i64, slug: &str) -> Self {
        PartyMember {
            id: -1,
            telegram_id,
            slug: slug.to_string(),
            to_transfer: 0f64,
            expenses: Vec::new(),
        }
    }

    pub fn with_id(id: i64, telegram_id: i64, slug: &str) -> Self {
        PartyMember {
            id,
            telegram_id,
            slug: slug.to_string(),
            to_transfer: 0f64,
            expenses: Vec::new(),
        }
    }

    pub fn with_expenses(
        id: PartyMemberId,
        telegram_id: i64,
        slug: &str,
        expenses: Vec<MemberExpense>,
    ) -> Self {
        PartyMember {
            id,
            telegram_id,
            slug: slug.to_string(),
            to_transfer: 0f64,
            expenses,
        }
    }

    pub fn id(&self) -> PartyMemberId {
        self.id
    }

    pub fn slug(&self) -> &str {
        &self.slug
    }

    pub fn telegram_id(&self) -> i64 {
        self.telegram_id
    }

    pub fn spent(&self) -> f64 {
        self.expenses
            .iter()
            .fold(self.to_transfer, |sum, item| sum + item.expense)
    }

    pub fn add_to_transfer(&mut self, amount: f64) {
        self.to_transfer += amount;
    }
}

#[derive(Debug, PartialEq)]
pub struct Transfer {
    from_id: String,
    to_id: String,
    amount: f64,
}

struct MembersSplit {
    creditors: Vec<String>,
    debtors: Vec<String>,
}

/// Defines every possible invariant for state machine implemented in Party struct behavior.
///
/// Possible states:
/// - Collecting: Users add expenses via commands or a Mini App.
/// - Locked: The host locks the party. No more expenses. Bot calculates and publishes the settlement plan.
/// - Settled: Users confirm payments. Bot tracks who has paid whom
pub enum PartyState {
    Collecting,
    Locked,
    Settled,
}

#[derive(Error, Debug)]
pub enum PartyError {
    #[error("invalid state transition")]
    InvalidTransition,
    #[error("party locked")]
    PartyLocked,
    #[error("already settled")]
    AlreadySettled,
    #[error("member already exists")]
    MemberAlreadyExists,
}

pub type PartyId = i64;

pub struct Party {
    id: PartyId,
    chat_id: i64,
    members: HashMap<String, PartyMember>,
    state: PartyState,
}

impl Party {
    pub fn new(chat_id: i64) -> Self {
        Party {
            id: -1,
            chat_id,
            state: PartyState::Collecting,
            members: HashMap::new(),
        }
    }

    pub fn from_raw(id: PartyId, chat_id: i64, members: Vec<PartyMember>, state: &str) -> Self {
        let state = match state {
            "locked" => PartyState::Locked,
            "settled" => PartyState::Settled,
            _ => PartyState::Collecting,
        };
        let members: HashMap<String, PartyMember> =
            members.into_iter().map(|m| (m.slug.clone(), m)).collect();
        Party {
            id,
            chat_id,
            members,
            state,
        }
    }

    pub fn get_member(&self, slug: PartyMemberSlug) -> Option<&PartyMember> {
        self.members.get(&slug)
    }

    pub fn id(&self) -> i64 {
        self.id
    }

    pub fn chat_id(&self) -> i64 {
        self.chat_id
    }

    pub fn state_str(&self) -> &str {
        match self.state {
            PartyState::Collecting => "collecting",
            PartyState::Locked => "locked",
            PartyState::Settled => "settled",
        }
    }

    fn get_member_or_create(&mut self, member_id: &str) -> &mut PartyMember {
        self.members
            .entry(member_id.to_string())
            .or_insert_with(|| PartyMember::new(0, member_id))
    }

    fn spent_avg(&self) -> f64 {
        let members_len = self.members.len();
        if members_len == 0 {
            return 0f64;
        }

        let total_spent: f64 = self
            .members
            .values()
            .fold(0f64, |sum, member| sum + member.spent());
        total_spent / members_len as f64
    }

    fn split_members(&self) -> MembersSplit {
        let mut creditors = Vec::new();
        let mut debtors = Vec::new();

        let avg = self.spent_avg();

        for member in self.members.values() {
            let member_spent: f64 = member.spent();
            let balance = member_spent - avg;
            if balance > 0f64 {
                creditors.push(member.slug.clone());
            } else {
                debtors.push(member.slug.clone());
            }
        }

        MembersSplit { creditors, debtors }
    }

    pub fn add_member(&mut self, member_id: &str) -> Result<(), PartyError> {
        match self.state {
            PartyState::Locked => return Err(PartyError::PartyLocked),
            PartyState::Settled => return Err(PartyError::AlreadySettled),
            _ => {}
        };

        if self.members.contains_key(member_id) {
            return Err(PartyError::MemberAlreadyExists);
        }

        self.members
            .insert(member_id.to_string(), PartyMember::new(0, member_id));

        Ok(())
    }

    pub fn add_expense(
        &mut self,
        member_id: &str,
        expense: MemberExpense,
    ) -> Result<(), PartyError> {
        match self.state {
            PartyState::Locked => return Err(PartyError::PartyLocked),
            PartyState::Settled => return Err(PartyError::AlreadySettled),
            _ => {}
        };

        let member = self.get_member_or_create(member_id);
        member.expenses.push(expense);

        Ok(())
    }

    pub fn evaluate(&mut self) -> Vec<Transfer> {
        let members_split = self.split_members();
        let debtors = members_split.debtors;
        let creditors = members_split.creditors;
        let mut transfers: Vec<Transfer> = Vec::new();

        let avg = self.spent_avg();

        let mut i = 0usize;
        let mut j = 0usize;
        while let (Some(debtor_id), Some(creditor_id)) = (debtors.get(i), creditors.get(j)) {
            let [debtor, creditor] = self.members.get_disjoint_mut([debtor_id, creditor_id]);
            let debtor = debtor.unwrap();
            let creditor = creditor.unwrap();

            let debt = (debtor.spent() - avg).abs();
            let credit = (creditor.spent() - avg).abs();

            let amount = match debt < credit {
                true => debt,
                false => credit,
            };

            transfers.push(Transfer {
                from_id: debtor_id.clone(),
                to_id: creditor_id.clone(),
                amount,
            });

            debtor.add_to_transfer(amount);
            creditor.add_to_transfer(-amount);

            let debt = debtor.spent() - avg;
            let credit = creditor.spent() - avg;
            if debt == 0f64 {
                i += 1;
            }
            if credit == 0f64 {
                j += 1;
            }
        }

        transfers
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn members_equalized_expenses_2() {
        let mut party = Party::new(0);

        party
            .add_expense(
                "alice",
                MemberExpense {
                    expense: 200f64,
                    description: None,
                },
            )
            .unwrap();
        party
            .add_expense(
                "alice",
                MemberExpense {
                    expense: 300f64,
                    description: None,
                },
            )
            .unwrap();
        party
            .add_expense(
                "bob",
                MemberExpense {
                    expense: 300f64,
                    description: None,
                },
            )
            .unwrap();

        let transfers = party.evaluate();
        assert_eq!(transfers.len(), 1);
        assert_eq!(
            Transfer {
                from_id: "bob".to_string(),
                to_id: "alice".to_string(),
                amount: 100f64
            },
            transfers[0]
        );
    }

    #[test]
    fn members_equalized_expenses_3() {
        let mut party = Party::new(0);

        party
            .add_expense(
                "alice",
                MemberExpense {
                    expense: 200f64,
                    description: None,
                },
            )
            .unwrap();
        party
            .add_expense(
                "bob",
                MemberExpense {
                    expense: 300f64,
                    description: None,
                },
            )
            .unwrap();
        party
            .add_expense(
                "charlie",
                MemberExpense {
                    expense: 400f64,
                    description: None,
                },
            )
            .unwrap();

        let transfers = party.evaluate();
        assert_eq!(transfers.len(), 1);
        assert_eq!(
            Transfer {
                from_id: "alice".to_string(),
                to_id: "charlie".to_string(),
                amount: 100f64
            },
            transfers[0]
        );
    }
}
