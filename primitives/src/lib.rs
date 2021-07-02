#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;
pub mod evm;

pub use currency::{CurrencyId, TokenSymbol};

#[cfg(test)]
mod tests;
