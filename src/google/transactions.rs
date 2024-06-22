//! Get transactions from a Google Sheet.

use chrono::NaiveDate;
use serde::Deserialize;
use serde_json::value::Value;

use crate::error::AppError as Error;

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
    pub category_split: Option<Vec<CategorySplit>>,
}

#[derive(Debug, Deserialize)]
pub struct CategorySplit {
    pub category: String,
    pub amount: f64,
}

impl GoogleSheet {
    pub async fn transactions(
        &self,
        sheet_details: &SheetDetails,
    ) -> Result<Option<Vec<Transaction>>, Error> {
        let range = format!("{}!A:P", sheet_details.name);

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
                category_split: parse_category_split(row.get(15))?,
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

fn parse_category_split(input: Option<&Value>) -> Result<Option<Vec<CategorySplit>>, Error> {
    match input {
        Some(Value::String(s)) => {
            // Split the string by commas
            let splits: Vec<&str> = s.split(',').collect();
            let mut category_splits = Vec::new();

            // Process each category split
            for split in splits {
                // Split by the colon to separate category and value
                let parts: Vec<&str> = split.split(':').collect();
                if parts.len() == 2 {
                    // Extract category and value
                    let category = parts[0].to_string().trim().to_string();
                    match parts[1].parse::<f64>() {
                        Ok(amount) => category_splits.push(CategorySplit { category, amount }),
                        Err(_) => {
                            return Err(Error::CategoryParseError(format!(
                                "Invalid value for category '{}'",
                                parts[0]
                            )))
                        }
                    }
                } else {
                    return Err(Error::CategoryParseError("Invalid format".to_string()));
                }
            }

            // Return the vector of CategorySplit
            Ok(Some(category_splits))
        }
        Some(_) => Ok(None), // Return None if the input value is not a string
        None => Ok(None),    // Return None if the input is None
    }
}

// -- Tests --------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string() {
        let value = Some(Value::String("test".to_string()));
        assert_eq!(parse_string(value.as_ref()), Some("test".to_string()));

        let value = Some(Value::String("".to_string()));
        assert_eq!(parse_string(value.as_ref()), None);

        let value = None;
        assert_eq!(parse_string(value), None);
    }

    #[test]
    fn test_parse_float() {
        let value = Value::String("1.23".to_string());
        assert_eq!(parse_float(value), 1.23);
    }

    #[test]
    fn test_parse_date() {
        let value = Value::String("01/02/2021".to_string());
        assert_eq!(
            parse_date(value),
            NaiveDate::from_ymd_opt(2021, 2, 1).unwrap()
        );
    }

    #[test]
    fn test_parse_category_split() {
        let value = Some(Value::String(
            "category1:1.23,category2:4.56, category3:-7.89".to_string(),
        ));
        let result = parse_category_split(value.as_ref()).unwrap().unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[2].category, "category3".to_string());
        assert_eq!(result[2].amount, -7.89);
    }

    #[test]
    fn test_parse_category_split_none() {
        let value = None;
        let result = parse_category_split(value.as_ref()).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn test_parse_category_split_err() {
        let value = Some(Value::String(
            "category1:1.23,category2:4.56,category3-7.89".to_string(),
        ));
        let result = parse_category_split(value.as_ref());

        assert!(result.is_err());
    }
}
