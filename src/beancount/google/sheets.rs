//! Gets sheets from a Google Sheets Account.

use crate::error::AppError as Error;

use super::GoogleSheet;

impl GoogleSheet {
    pub async fn sheets(&self) -> Result<Option<Vec<String>>, Error> {
        let result = self
            .hub
            .spreadsheets()
            .get(&self.account.sheet_id)
            .doit()
            .await?;

        let sheets = match result.1.sheets {
            Some(sheets) => Some(
                sheets
                    .iter()
                    .filter_map(|sheet| sheet.properties.as_ref().and_then(|p| p.title.as_ref()))
                    .map(|title| title.to_string())
                    .collect(),
            ),
            None => None,
        };

        Ok(sheets)
    }
}
