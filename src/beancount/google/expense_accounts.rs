//! Gets the list of expense accounts for a Google Account Sheet.

use std::collections::HashSet;

use super::GoogleSheet;

use crate::error::AppError as Error;

impl GoogleSheet {
    /// Get the list of expense accounts for a Google Account Sheet.
    pub async fn expense_accounts(&self) -> Result<Vec<String>, Error> {
        let transactions = match &self.transactions {
            Some(transactions) => transactions,
            None => return Ok(vec![]),
        };

        let filter_categories = vec![
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
    use crate::beancount::google::config::load_sheets;

    use super::*;

    #[tokio::test]
    #[ignore]
    async fn expense_accounts() {
        let accounts = load_sheets().unwrap();
        let account = accounts[0].clone();
        let sheet = GoogleSheet::new(account).await.unwrap();

        let expense_accounts = sheet.expense_accounts().await.unwrap();
        assert!(expense_accounts.len() > 0);
    }
}
