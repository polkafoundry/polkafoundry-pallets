use super::*;
use crate as oracle;

use frame_support::{
	construct_runtime, parameter_types,
};
use sp_runtime::{
	testing::{Header},
	traits::{IdentityLookup, BlakeTwo256},
};
use sp_std::cell::RefCell;
use sp_core::H256;

/// The AccountId alias in this test module.
pub(crate) type AccountId = u64;
pub(crate) type Balance = u64;
type Key = u32;
type Value = u32;

pub const POLKAFOUNDRY: u64 = 1;
pub const ALICE: u64 = 2;
pub const BOB: u64 = 3;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(
			frame_support::weights::constants::WEIGHT_PER_SECOND * 2
		);
}

impl frame_system::Config for Test {
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type BlockWeights = ();
	type BlockLength = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type DbWeight = ();
	type BaseCallFilter = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
	type OnSetCode = ();
}

parameter_types! {
	pub const ExistentialDeposit: u64 = 1;
}

impl pallet_balances::Config for Test {
	type Balance = Balance;
	type DustRemoval = ();
	type Event = Event;
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type MaxLocks = ();
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type WeightInfo = ();
}

thread_local! {
	static TIME: RefCell<u32> = RefCell::new(0);
}

pub struct Timestamp;
impl Time for Timestamp {
	type Moment = u32;

	fn now() -> Self::Moment {
		TIME.with(|v| *v.borrow())
	}
}

impl Timestamp {
	pub fn set_timestamp(val: u32) {
		TIME.with(|v| *v.borrow_mut() = val);
	}
}

parameter_types! {
	pub const MinimumCount: u32 = 3;
	pub const ExpiresIn: u32 = 600;
	pub const Fee: u64 = 100;
}

impl Config for Test {
	type Event = Event;
	type CombineData = DefaultCombineData<Self, MinimumCount, ExpiresIn>;
	type Time = Timestamp;
	type FeedKey = Key;
	type FeedValue = Value;
	type Currency = Balances;
	type OracleFee = Fee;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Oracle: oracle::{Pallet, Storage, Call, Config<T>, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
	}
);

pub struct ExtBuilder {
	feeders: Vec<AccountId>,
}

impl Default for ExtBuilder {
	fn default() -> Self {
		Self {
			feeders: vec![],
		}
	}
}

impl ExtBuilder {
	pub fn feeders(mut self, feeders: Vec<AccountId>) -> Self {
		self.feeders = feeders;
		self
	}

	pub fn alice_is_feeder(mut self) -> Self {
		self.feeders = vec![ALICE];
		self
	}

	pub fn build(self) -> sp_io::TestExternalities {
		let mut t = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();

		oracle::GenesisConfig::<Test> {
			feeders: self.feeders,
			_phantom: Default::default()
		}
			.assimilate_storage(&mut t)
			.unwrap();

		let mut ext = sp_io::TestExternalities::new(t);
		ext.execute_with(|| {
			System::set_block_number(1);
			Timestamp::set_timestamp(12345);
		});
		ext
	}

	pub fn build_and_execute(self, test: impl FnOnce() -> ()) {
		let mut ext = self.build();
		ext.execute_with(test);
	}
}

