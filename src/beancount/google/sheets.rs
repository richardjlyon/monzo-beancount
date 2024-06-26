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

        let sheets = result.1.sheets.map(|sheets| {
            sheets
                .into_iter()
                .filter_map(|sheet| sheet.properties.and_then(|p| p.title))
                .map(|title| title.to_string())
                .collect()
        });

        Ok(sheets)
    }
}
