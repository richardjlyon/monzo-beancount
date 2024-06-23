//! Beancount export
//!
//! This module generates a set of accounts in Beancount format from financial information
//! stored in the database.

pub mod account;
pub mod config;
pub mod directive;
pub mod transaction;

use config::BeanSettings;

/// A struct representing a Beancount file
pub struct Beancount {
    pub settings: BeanSettings,
}
