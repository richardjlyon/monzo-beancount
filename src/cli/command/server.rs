//! Run a server to periodically fetch transactions from Monzo and write them to a
//! Beancount file.

use crate::{beancount::Beancount, error::AppError as Error};
use tokio::signal;
use tokio::time::{self, Duration};

pub async fn server() -> Result<(), Error> {
    if let Err(e) = generate_bean_periodically().await {
        eprintln!("Error: {:?}", e);
    }
    Ok(())
}

async fn generate_bean_periodically() -> Result<(), Error> {
    let mut interval = time::interval(Duration::from_secs(10));
    let bean = Beancount::from_user_config()?;

    loop {
        println!("->> refreshing...");
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = bean.generate().await {
                    eprintln!("Error generating bean: {:?}", e);
                }
            }
            _ = signal::ctrl_c() => {
                println!("Received Ctrl-C, shutting down.");
                break;
            }
        }
    }

    Ok(())
}
