#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;
pub mod evm;
use codec::{Encode, Decode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::RuntimeDebug;

pub use currency::{CurrencyId, TokenSymbol};

#[cfg(test)]
mod tests;

#[derive(Encode, Decode, Eq, PartialEq, Copy, Clone, RuntimeDebug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum DataProviderId {
	Combined = 0,
	PolkaFoundry = 1,
	Concrete = 2,
	All = 3,
}

