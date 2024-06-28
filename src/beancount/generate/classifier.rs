//! Functionality for classifying a transaction.

use std::collections::HashSet;

use crate::beancount::account::Account as BeancountAccount;
use crate::beancount::google::transactions::Transaction as GoogleTransaction;
use crate::error::AppError as Error;

#[derive(Debug, PartialEq)]
pub(crate) enum Classification {
    IncomeGeneral,
    IncomeAccount(BeancountAccount),
    Savings,
    TransferOpeningBalance,
    TransferPot,
    TransferAsset(BeancountAccount),
    // TransferExpense,
}

pub(crate) fn classify_transaction(
    asset_accounts: &Vec<BeancountAccount>,
    income_accounts: &Vec<BeancountAccount>,
    tx: &GoogleTransaction,
) -> Result<Option<Classification>, Error> {
    match tx.category.as_str() {
        "Income" => {
            if is_income_account(income_accounts, &tx.name) {
                let income_account = income_account_finder(income_accounts, &tx.name)
                    .expect("Failed to get institution name");
                return Ok(Some(Classification::IncomeAccount(income_account)));
            }

            if is_custom_transfer(tx) {
                return Ok(Some(Classification::TransferOpeningBalance));
            }

            Ok(Some(Classification::IncomeGeneral))
        }

        "Savings" => Ok(Some(Classification::Savings)),

        "Transfers" => {
            if tx.payment_type == "Pot transfer" {
                return Ok(Some(Classification::TransferPot));
            }

            if is_asset_account(asset_accounts, &tx.name) {
                let asset_account = asset_account_finder(asset_accounts, &tx.name)
                    .expect("Failed to get asset name");
                return Ok(Some(Classification::TransferAsset(asset_account)));
            }

            Ok(Some(Classification::TransferOpeningBalance))
        }
        _ => Ok(None),
    }
}

fn is_custom_transfer(tx: &GoogleTransaction) -> bool {
    tx.notes
        .as_ref()
        .unwrap_or(&String::new())
        .starts_with("Account Switch")
}

fn is_asset_account(accounts: &Vec<BeancountAccount>, account: &str) -> bool {
    let asset_accounts = get_filtered_asset_accounts(accounts).unwrap();
    asset_accounts.iter().any(|a| a.account == account)
}

fn asset_account_finder(
    accounts: &Vec<BeancountAccount>,
    account_to_find: &str,
) -> Option<BeancountAccount> {
    let filtered_assets = get_filtered_asset_accounts(accounts).unwrap();
    let matching_assets: Vec<BeancountAccount> = filtered_assets
        .iter()
        .filter(|&asset| asset.account == account_to_find)
        .cloned() // Use `cloned` to get ownership of the filtered assets
        .collect();
    match matching_assets.len() {
        1 => Some(matching_assets[0].clone()),
        _ => None,
    }
}

fn get_filtered_asset_accounts(
    assets: &Vec<BeancountAccount>,
) -> Result<Vec<BeancountAccount>, Error> {
    // FIXME: This is a temporary solution to filter out unwanted accounts. Refactor to use config data.
    let unwanted_accounts = ["Business", "Personal"];

    let unique_accounts: HashSet<BeancountAccount> = assets
        .iter()
        .filter(|a| !unwanted_accounts.contains(&a.account.as_str()))
        .cloned()
        .collect();

    Ok(unique_accounts.into_iter().collect())
}

fn is_income_account(accounts: &Vec<BeancountAccount>, account: &str) -> bool {
    let income_accounts = get_filtered_income_accounts(accounts).unwrap();
    income_accounts.iter().any(|a| a.account == account)
}

/// Find an income account in the Beancount configuration with the name `accouunt_to_find`.
fn income_account_finder(
    accounts: &Vec<BeancountAccount>,
    account_to_find: &str,
) -> Option<BeancountAccount> {
    let filtered_income = get_filtered_income_accounts(accounts).unwrap();
    let matching_income: Vec<BeancountAccount> = filtered_income
        .iter()
        .filter(|&income| income.account == account_to_find)
        .cloned() // Use `cloned` to get ownership of the filtered assets
        .collect();
    match matching_income.len() {
        1 => Some(matching_income[0].clone()),
        _ => None,
    }
}

/// Get the income accounts from config and remove the main accounts.
fn get_filtered_income_accounts(
    incomes: &Vec<BeancountAccount>,
) -> Result<Vec<BeancountAccount>, Error> {
    // FIXME: This is a temporary solution to filter out unwanted accounts. Refactor to use config data.
    let unwanted_accounts = ["Business", "Personal"];

    let unique_accounts: HashSet<BeancountAccount> = incomes
        .iter()
        .filter(|a| !unwanted_accounts.contains(&a.account.as_str()))
        .cloned()
        .collect();

    Ok(unique_accounts.into_iter().collect())
}

// -- Tests ----------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use chrono::NaiveDate;

    use crate::beancount::Beancount;

    use super::*;

    #[test]
    fn should_classify_income_general() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AfJaPxueG5vGjc9LqT".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Faster payment".to_string(),
            name: "MPB EUROPE".to_string(),
            category: "Income".to_string(),
            amount: 19100,
            currency: "GBP".to_string(),
            local_amount: 19100,
            local_currency: "GBP".to_string(),
            notes: Some("MPB TX-UK-7836178".to_string()),
            description: Some("MPB TX-UK-7836178".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert
        assert_eq!(classification, Some(Classification::IncomeGeneral));
    }

    #[test]
    fn should_classify_income_account_bp() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AePXivwOdKv8HMbyxm".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Bacs (Direct Credit)".to_string(),
            name: "Bp Pension Trustee".to_string(),
            category: "Income".to_string(),
            amount: 225945,
            currency: "GBP".to_string(),
            local_amount: 225945,
            local_currency: "GBP".to_string(),
            notes: Some("BPF0021628".to_string()),
            description: Some("BPF0021628".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert

        if let Some(Classification::IncomeAccount(income_account)) = classification {
            assert_eq!(income_account.account, "Bp Pension Trustee".to_string());
        } else {
            panic!("Expected IncomeAccount but got {:?}", classification);
        }
    }

    #[test]
    fn should_classify_income_account_airbnb() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AhhITbH5KFk4tUplBr".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Bacs (Direct Credit)".to_string(),
            name: "Citibank".to_string(),
            category: "Income".to_string(),
            amount: 27956,
            currency: "GBP".to_string(),
            local_amount: 27956,
            local_currency: "GBP".to_string(),
            notes: Some("AIRBNB PAYMENTS UK".to_string()),
            description: Some("AIRBNB PAYMENTS UK".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert

        if let Some(Classification::IncomeAccount(income_account)) = classification {
            assert_eq!(income_account.account, "Citibank".to_string());
        } else {
            panic!("Expected IncomeAccount but got {:?}", classification);
        }
    }

    #[test]
    fn should_classify_savings() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AdV0balgmGFiUDRI4A".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Faster payment".to_string(),
            name: "Richard Lyon".to_string(),
            category: "Savings".to_string(),
            amount: -10000,
            currency: "GBP".to_string(),
            local_amount: -10000,
            local_currency: "GBP".to_string(),
            notes: Some("???".to_string()),
            description: Some("Richard Lyon".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert
        assert_eq!(classification, Some(Classification::Savings));
    }

    #[test]
    #[ignore = "fix beancount fixture"]
    fn should_classify_transfer_opening_balance() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AdUzArSgVGj1ntv0eA".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Faster payment".to_string(),
            name: "Richard Lyon".to_string(),
            category: "Transfers".to_string(),
            amount: 450087,
            currency: "GBP".to_string(),
            local_amount: 450087,
            local_currency: "GBP".to_string(),
            notes: Some("Transfer in from Starling".to_string()),
            description: Some("Monzo-BHKTM".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert
        assert_eq!(classification, Some(Classification::TransferOpeningBalance));
    }

    #[test]
    fn should_classify_transfer_pot() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AdRKEtYzx4cRduaFEX".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Pot transfer".to_string(),
            name: "Essential Fixed Pot".to_string(),
            category: "Transfers".to_string(),
            amount: -10000,
            currency: "GBP".to_string(),
            local_amount: -10000,
            local_currency: "GBP".to_string(),
            notes: Some("To fund pot".to_string()),
            description: None,
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert
        assert_eq!(classification, Some(Classification::TransferPot));
    }

    #[test]
    fn should_classify_transfer_asset_nsi() {
        // Arrange
        let (asset_accounts, income_accounts) = get_test_accounts();
        let tx = GoogleTransaction {
            id: "tx_0000AdVRzCp69ZxOqfBdXl".to_string(),
            date: NaiveDate::from_ymd_opt(2021, 8, 1).unwrap(),
            payment_type: "Faster payment".to_string(),
            name: "NSI Premium Bonds".to_string(),
            category: "Transfers".to_string(),
            amount: -10000,
            currency: "GBP".to_string(),
            local_amount: -10000,
            local_currency: "GBP".to_string(),
            notes: Some("520344086".to_string()),
            description: Some("520344086".to_string()),
            category_split: None,
        };

        // Act
        let classification = classify_transaction(&asset_accounts, &income_accounts, &tx).unwrap();

        // Assert
        if let Some(Classification::TransferAsset(asset)) = classification {
            assert_eq!(asset.account, "NSI Premium Bonds".to_string());
        } else {
            panic!("Expected TransferAsset classification");
        }
    }

    fn get_test_accounts() -> (Vec<BeancountAccount>, Vec<BeancountAccount>) {
        let data_dir = PathBuf::from("/Users/richardlyon/dev/rust-monzo-beancount/data");
        let bc = Beancount::with_data_dir(data_dir).expect("Failed to create Beancount instance");
        let asset_accounts = bc.user_settings.assets.clone().unwrap();
        let income_accounts = bc.user_settings.income.clone().unwrap();

        (asset_accounts, income_accounts)
    }
}
