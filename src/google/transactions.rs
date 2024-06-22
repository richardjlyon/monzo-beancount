//! Get transactions from a Google Sheet.

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::value::Value;

use crate::error::AppError;

use super::{GoogleSheet, SheetDetails};

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub id: String,
    pub date: NaiveDate,
    pub payment_type: String,
    pub name: String,
    pub category: String,
    pub amount: f64,
    pub currency: String,
    pub local_amount: f64,
    pub local_currency: String,
    pub notes: Option<String>,
    pub description: Option<String>,
    pub category_split: Option<String>,
}

impl GoogleSheet {
    pub async fn transactions(
        &self,
        sheet_details: &SheetDetails,
    ) -> Result<Option<Vec<Transaction>>, AppError> {
        let range = format!("{}!A:O", sheet_details.name);

        let result = self
            .hub
            .spreadsheets()
            .values_get(&sheet_details.id, &range)
            .doit()
            .await?;

        // println!("Raw API Response: {:#?}", result);

        let values = match result.1.values {
            Some(values) => values,
            None => return Ok(None),
        };

        let mut transactions: Vec<Transaction> = Vec::new();

        for row in values.iter().skip(1) {
            let transaction = Transaction {
                id: parse_string(row.get(0)).unwrap_or_default(),
                date: parse_date(row[1].clone()),
                payment_type: parse_string(row.get(3)).unwrap_or_default(),
                name: parse_string(row.get(4)).unwrap_or_default(),
                category: parse_string(row.get(6)).unwrap_or_default(),
                amount: parse_float(row[7].clone()),
                currency: parse_string(row.get(8)).unwrap_or_default(),
                local_amount: parse_float(row[9].clone()),
                local_currency: parse_string(row.get(10)).unwrap_or_default(),
                notes: parse_string(row.get(11)),
                description: parse_string(row.get(14)),
                category_split: parse_string(row.get(15)),
            };
            transactions.push(transaction);
        }

        Ok(Some(transactions))
    }
}

fn parse_string(value: Option<&Value>) -> Option<String> {
    value
        .and_then(|v| v.as_str()) // Try to get the &str from Value
        .filter(|s| !s.is_empty()) // Check if the string is not empty
        .map(|s| s.to_string()) // Convert the &str to String
}

fn parse_float(amount: Value) -> f64 {
    let amount_str = amount.to_string().replace("\"", "");
    amount_str.parse::<f64>().unwrap()
}

fn parse_date(date: Value) -> NaiveDate {
    let date_str = date.to_string().replace("\"", "");
    NaiveDate::parse_from_str(&date_str, "%d/%m/%Y").unwrap()
}
