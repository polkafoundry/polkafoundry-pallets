#![cfg_attr(not(feature = "std"), no_std)]

use codec::Codec;
use sp_std::vec::Vec;

sp_api::decl_runtime_apis! {
	pub trait OracleApi<ProviderId, AccountId, Key, Value> where
		ProviderId: Codec,
		AccountId: Codec,
		Key: Codec,
		Value: Codec,
	{
		fn get(provider_id: ProviderId, key: Key) -> Option<Value>;
		fn get_polkafoundry(provider_id: ProviderId, key: Key, feeder: AccountId) -> Option<Value>;
		fn get_concrete(provider_id: ProviderId, key: Key, feeder: AccountId) -> Option<Value>;
		fn get_all_value(provider_id: ProviderId, key: Key) -> Vec<(Key, Option<Value>)>;
	}
}
