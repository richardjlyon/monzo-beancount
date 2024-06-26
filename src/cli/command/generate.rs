//! (Re)generate the main beancount file.

use crate::{beancount::Beancount, error::AppError as Error};

pub async fn generate() -> Result<(), Error> {
    let bean = Beancount::from_config()?;
    bean.generate().await?;

    Ok(())
}
