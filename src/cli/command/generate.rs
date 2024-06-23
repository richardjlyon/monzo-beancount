//! Generate beancount files command.
use crate::{beancount::Beancount, error::AppError as Error};

pub async fn generate() -> Result<(), Error> {
    let bean = Beancount::from_config()?;
    bean.generate().await?;

    Ok(())
}
