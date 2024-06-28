//! Run a server to periodically fetch transactions from Monzo and write them to a
//! Beancount file.

use crate::{beancount::Beancount, error::AppError as Error};
use tokio::signal;
use tokio::time::{self, Duration};

pub async fn server(beancount: &Beancount, interval_secs: u64) -> Result<(), Error> {
    if let Err(e) = generate_bean_periodically(beancount, interval_secs).await {
        eprintln!("Error: {:?}", e);
    }
    Ok(())
}

async fn generate_bean_periodically(
    beancount: &Beancount,
    interval_secs: u64,
) -> Result<(), Error> {
    let mut interval = time::interval(Duration::from_secs(interval_secs));

    loop {
        println!("->> refreshing...");
        tokio::select! {
            _ = interval.tick() => {
                if let Err(e) = beancount.generate().await {
                    eprintln!("Error generating beanfile: {:?}", e);
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
