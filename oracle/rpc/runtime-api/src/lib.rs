#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait OracleApi<AccountId, Key, Value> where
		AccountId: Codec,
		Key: Codec,
		Value: Codec,
	{
		fn get(key: Key) -> Option<Value>;
		fn get_polkafoundry(key: Key) -> Option<Value>;
		fn get_concrete(key: Key, feeder: AccountId) -> Option<Value>;
		fn get_all_values() -> Vec<(Key, Option<Value>)>;
	}
}
