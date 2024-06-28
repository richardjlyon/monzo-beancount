//! (Re)generate the main beancount file.

use crate::{beancount::Beancount, error::AppError as Error};

pub async fn generate(beancount: &Beancount) -> Result<(), Error> {
    beancount.generate().await?;

    Ok(())
}
