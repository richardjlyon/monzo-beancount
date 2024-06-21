//! Get transactions from a Google Sheet.

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::value::Value;

use crate::errors::AppError;

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
    // notes: String,
    // description: String,
}

impl GoogleSheet {
    pub async fn transactions(
        &self,
        sheet_details: &SheetDetails,
    ) -> Result<Option<Vec<Transaction>>, AppError> {
        let range = format!("{}!A:P", sheet_details.name);

        let result = self
            .hub
            .spreadsheets()
            .values_get(&sheet_details.id, &range)
            .doit()
            .await?;

        let values = match result.1.values {
            Some(values) => values,
            None => return Ok(None),
        };

        let mut rows: Vec<Transaction> = Vec::new();

        for row in values.iter().skip(1) {
            let data = Transaction {
                id: parse_string(row[0].clone()),
                date: parse_date(row[1].clone()),
                payment_type: parse_string(row[3].clone()),
                name: parse_string(row[4].clone()),
                category: parse_string(row[6].clone()),
                amount: parse_float(row[7].clone()),
                currency: parse_string(row[8].clone()),
                local_amount: parse_float(row[9].clone()),
                local_currency: parse_string(row[10].clone()),
                // notes: parse_string(row[11].clone()),
                // description: parse_string(row[14].clone()),
            };
            rows.push(data);
        }

        Ok(Some(rows))
    }
}

fn parse_string(id: Value) -> String {
    id.to_string().replace("\"", "")
}

fn parse_float(amount: Value) -> f64 {
    let amount_str = amount.to_string().replace("\"", "");
    amount_str.parse::<f64>().unwrap()
}

fn parse_date(date: Value) -> NaiveDate {
    let date_str = date.to_string().replace("\"", "");
    NaiveDate::parse_from_str(&date_str, "%d/%m/%Y").unwrap()
}
