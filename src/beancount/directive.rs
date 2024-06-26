//! Represents a Beancount [directive](https://beancount.github.io/docs/beancount_language_syntax.html#directives_1) and handles formatting.

use chrono::NaiveDate;
use convert_case::{Case, Casing};

use super::{account::Account, transaction::Transaction as BeanTransaction};

type Comment = String;

/// Represents a Beancount directive.
#[derive(Debug)]
pub enum Directive {
    Option(String, String),
    Include(String),
    Comment(String),
    Open(NaiveDate, Account, Option<Comment>),
    Close(NaiveDate, Account, Option<Comment>),
    Transaction(BeanTransaction),
    Balance(NaiveDate, Account),
}

impl Directive {
    #[must_use]
    pub fn to_formatted_string(&self) -> String {
        let account_width = 50;
        match self {
            Directive::Include(file) => format!("include \"{}\"\n", file),

            Directive::Option(key, value) => format!("option \"{}\" \"{}\"\n", key, value),

            Directive::Comment(comment) => format!("\n* {}\n\n", comment.to_case(Case::Title)),

            Directive::Open(date, account, comment) => {
                let currency = &account.country;
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                return format!(
                    "{}{} open {:account_width$} {}\n",
                    comment,
                    date,
                    account.to_string(),
                    currency
                );
            }

            Directive::Transaction(transaction) => {
                format!("{}\n", transaction.to_formatted_string())
            }

            Directive::Close(date, account, comment) => {
                let comment = match comment {
                    Some(c) => format!("; {c}.\n"),
                    None => String::new(),
                };
                format!(
                    "{}{} close {:account_width$}\n",
                    comment,
                    date,
                    account.to_string(),
                )
            }

            Directive::Balance(_date, _account) => {
                todo!()
            }
        }
    }
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::beancount::account::AccountType;

    use super::*;

    #[test]
    fn open_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
            transaction_id: None,
        };
        // Act
        let directive = Directive::Open(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 open Assets:GBP:Monzo:Personal                          GBP\n"
        );
    }

    #[test]
    fn open_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
            transaction_id: None,
        };
        let comment = Some("Initial Deposit".to_string());
        // Act
        let directive = Directive::Open(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; Initial Deposit.\n2024-06-13 open Assets:GBP:Monzo:Personal                          GBP\n"
        );
    }

    #[test]
    fn close_directive() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
            transaction_id: None,
        };
        // Act
        let directive = Directive::Close(date, account, None);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "2024-06-13 close Assets:GBP:Monzo:Personal                         \n"
        );
    }

    #[test]
    fn close_directive_comment() {
        // Arrange
        let date = NaiveDate::from_ymd_opt(2024, 6, 13).unwrap();
        let account = Account {
            account_type: AccountType::Assets,
            country: "GBP".to_string(),
            institution: "Monzo".to_string(),
            account: "Personal".to_string(),
            sub_account: None,
            transaction_id: None,
        };
        let comment = Some("To Close".to_string());
        // Act
        let directive = Directive::Close(date, account, comment);
        // Assert
        assert_eq!(
            directive.to_formatted_string(),
            "; To Close.\n2024-06-13 close Assets:GBP:Monzo:Personal                         \n"
        );
    }
}
