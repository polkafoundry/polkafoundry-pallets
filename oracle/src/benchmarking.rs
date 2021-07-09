#![cfg(feature = "runtime-benchmarks")]

use super::*;
use crate::Pallet as Oracle;

use frame_system::RawOrigin;
use frame_support::{assert_ok};
pub use frame_benchmarking::{
	benchmarks, account, whitelisted_caller, whitelist_account, impl_benchmark_test_suite,
};
const USER_SEED: u32 = 999666;

benchmarks! {
	elect_feeder {
		let feeder: T::AccountId = account("feeder", 0u32, USER_SEED);
	}: _(RawOrigin::Root, feeder.clone())
	verify {
		let mut feeders = <Feeders<T>>::get();
		assert_ok!(feeders.binary_search(&feeder));
	}
	set_fee {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Signed(caller.clone()), 1u32.into())
	verify {
		assert_eq!(<Fees<T>>::get(caller), 1u32.into());
	}
	remove_feeder {
		let caller: T::AccountId = whitelisted_caller();
	}: _(RawOrigin::Root, caller.clone())
	verify {
		let mut feeders = <Feeders<T>>::get();
		assert_eq!(feeders.binary_search(&caller), Err(0));
	}
}

impl_benchmark_test_suite!(
	Oracle,
	crate::mock::ExtBuilder::default().feeders(vec![frame_benchmarking::whitelisted_caller()]),
	crate::mock::Test,
	exec_name = build_and_execute
);
