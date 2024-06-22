//! Update transactions command

use std::fs::File;
use std::io::Write;

use crate::beancount::directive::Directive;
use crate::beancount::Beancount;
use crate::error::AppError as Error;

pub async fn update() -> Result<(), Error> {
    let mut directives: Vec<Directive> = Vec::new();

    let bean = Beancount::from_config()?;

    // -- Initialise the file system -----------------------------------------------------

    match bean.initialise_filesystem()? {
        Some(message) => println!("{}", message),
        None => {}
    };

    // -- Open Equity Accounts -----------------------------------------------------

    directives.push(Directive::Comment("equity accounts".to_string()));
    directives.extend(open_config_equity_accounts()?);

    // -- Open Asset Accounts --------------------------------------------------------------

    directives.push(Directive::Comment("asset accounts".to_string()));

    // -- Post Transactions---------------------------------------------------------

    directives.push(Directive::Comment("transactions".to_string()));

    // for tx in transactions {
    //     println!("{:#?}", tx);
    // }

    // -- Write directives to file -----------------------------------------------------

    let file_path = bean.settings.root_dir.join("report.beancount");
    let mut file = File::create(file_path)?;
    for d in directives {
        file.write_all(d.to_formatted_string().as_bytes())?;
    }

    Ok(())
}

fn open_config_equity_accounts() -> Result<Vec<Directive>, Error> {
    let bc = Beancount::from_config()?;
    let mut directives: Vec<Directive> = Vec::new();

    if let Some(equity_accounts) = bc.settings.equity {
        for equity in equity_accounts {
            directives.push(Directive::OpenEquity(bc.settings.start_date, equity, None));
        }
    }

    Ok(directives)
}
