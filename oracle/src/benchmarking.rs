use super::*;
use crate::Pallet as Oracle;

use frame_system::RawOrigin;
pub use frame_benchmarking::{
	benchmarks, account, whitelisted_caller, whitelist_account, impl_benchmark_test_suite,
};

const DOT: u32 = 1;
const KSM: u32 = 2;
const BTC: u32 = 3;
const ETH: u32 = 4;
const PKF_FEEDER: u64 = crate::mock::POLKAFOUNDRY;

const KEY: Vec<u32> = vec![DOT, KSM, BTC, ETH];

benchmarks! {
	feed_values {
		let mut values = vec![];
		for i in 0..KEY.len() - 1 {
			values.push((KEY[i as usize], 1));
		}

	}: _(RawOrigin::Signed(PKF_FEEDER), values)
	verify {
		assert_eq!(AllValue::<T>::len(), 4);
	}
}

impl_benchmark_test_suite!(
	Oracle,
	crate::mock::ExtBuilder::default().feeders(vec![PKF_FEEDER]),
	crate::mock::Test,
	exec_name = build_and_execute
);
