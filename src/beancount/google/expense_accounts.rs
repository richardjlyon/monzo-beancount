//! Gets the list of expense accounts for a Google Account Sheet.

use std::collections::HashSet;

use crate::error::AppError as Error;

use super::GoogleSheet;

impl GoogleSheet {
    /// Get the list of expense accounts for a Google Account Sheet.
    pub async fn expense_accounts(&self) -> Result<Vec<String>, Error> {
        let transactions = match &self.transactions {
            Some(transactions) => transactions,
            None => return Ok(vec![]),
        };

        let filter_categories = [
            "Income".to_string(),
            "Savings".to_string(),
            "Transfers".to_string(),
        ];

        let filtered_set: HashSet<_> = transactions
            .iter()
            .filter(|t| !filter_categories.contains(&t.category))
            .map(|t| t.category.clone())
            .collect();

        let mut expense_accounts: Vec<String> = filtered_set.into_iter().collect();
        expense_accounts.sort();

        Ok(expense_accounts)
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]

mod tests {
    use std::path::PathBuf;

    use crate::beancount::Beancount;

    use super::*;

    #[tokio::test]
    async fn expense_accounts() {
        let data_dir = PathBuf::from("/Users/richardlyon/dev/rust-monzo-beancount/data");
        let bc = Beancount::with_data_dir(data_dir).expect("Failed to create Beancount instance");
        let accounts = bc.user_settings.googlesheet_accounts.unwrap();

        let account = accounts[0].clone();
        let sheet = GoogleSheet::new(account).await.unwrap();

        let expense_accounts = sheet.expense_accounts().await.unwrap();
        assert!(!expense_accounts.is_empty());
    }
}
