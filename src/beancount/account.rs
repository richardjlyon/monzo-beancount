//! Represents a Beancount [account](https://beancount.github.io/docs/the_double_entry_counting_method.html#types-of-accounts) and handles formatting.
//!

use core::fmt;

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};

/// Represents permissable Beancount account types.
#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, strum_macros::Display)]
pub enum AccountType {
    Assets,
    Liabilities,
    Income,
    Expenses,
    Equity,
}

/// Represents a Beancount account.
///
#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
pub struct Account {
    pub(crate) account_type: AccountType,
    pub(crate) country: String,
    pub(crate) institution: String,
    pub(crate) account: String,
    pub(crate) sub_account: Option<String>,
    pub(crate) transaction_id: Option<String>,
}

impl fmt::Display for Account {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let label = match &self.sub_account {
            Some(sub_account) => format!(":{}", sub_account.to_case(Case::Pascal)),
            None => String::new(),
        };
        match &self.account_type {
            AccountType::Equity => {
                write!(
                    f,
                    "{}{}",
                    format!("{}", self.account_type),
                    format!(":{}", self.account.to_case(Case::Pascal)),
                )
            }
            _ => {
                write!(
                    f,
                    "{}{}{}{}{}",
                    format!("{}", self.account_type),
                    format!(":{}", self.country.to_case(Case::Upper)),
                    format!(":{}", self.institution.to_case(Case::Pascal)),
                    format!(":{}", self.account.to_case(Case::Pascal)),
                    format!("{}", label),
                )
            }
        }
    }
}
