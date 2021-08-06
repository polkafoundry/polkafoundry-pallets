use crate::{self as pallet_redkite, Config};
use frame_support::{assert_ok, construct_runtime, parameter_types, PalletId};

use frame_support::traits::GenesisBuild;
use orml_traits::parameter_type_with_key;
use sp_core::H256;
use sp_io;
use sp_runtime::{
	testing::Header,
	traits::{AccountIdConversion, BlakeTwo256, IdentityLookup},
};
use sp_std::convert::From;

pub type AccountId = u64;
pub type Balance = u128;
pub type Moment = u64;
type CurrencyId = u32;

parameter_types! {
	pub const BlockHashCount: u64 = 250;
	pub BlockWeights: frame_system::limits::BlockWeights =
		frame_system::limits::BlockWeights::simple_max(1024);
}

impl frame_system::Config for Test {
	type BaseCallFilter = ();
	type BlockWeights = ();
	type BlockLength = ();
	type Origin = Origin;
	type Index = u64;
	type Call = Call;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = AccountId;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = BlockHashCount;
	type DbWeight = ();
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<Balance>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type OnSetCode = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ();
}

parameter_types! {
	pub const ExistentialDeposit: u128 = 1;
	pub const MaxLocks: u32 = 100_000;
	pub const MaxReserves: u32 = 100_000;
}

impl pallet_balances::Config for Test {
	type MaxLocks = MaxLocks;
	type Balance = Balance;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ExistentialDeposit;
	type AccountStore = System;
	type WeightInfo = ();
	type MaxReserves = MaxReserves;
	type ReserveIdentifier = [u8; 8];
}

impl pallet_utility::Config for Test {
	type Event = Event;
	type Call = Call;
	type WeightInfo = ();
}

parameter_type_with_key! {
	pub ExistentialDeposits: |_currency_id: CurrencyId| -> Balance {
		Default::default()
	};
}

parameter_types! {
	pub DustAccount: AccountId = PalletId(*b"orml/dst").into_account();
}

impl orml_tokens::Config for Test {
	type Event = Event;
	type Balance = Balance;
	type Amount = i64;
	type CurrencyId = CurrencyId;
	type WeightInfo = ();
	type ExistentialDeposits = ExistentialDeposits;
	type OnDust = orml_tokens::TransferDust<Test, DustAccount>;
	type MaxLocks = MaxLocks;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 1000 / 2;
}

impl pallet_timestamp::Config for Test {
	/// A timestamp: milliseconds since the unix epoch.
	type Moment = Moment;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}

parameter_types! {
	pub const RedkitePalletId: PalletId = PalletId(*b"Redkite ");
}

impl Config for Test {
	type Event = Event;
	type PalletId = RedkitePalletId;
	type Currency = Balances;
	type MultiCurrency = Tokens;
	type Time = Timestamp;
}

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;
pub const INIT_BALANCE: u128 = 100_000_000;
pub const DEFAULT_TIER_DOVE: u128 = 500;
pub const DEFAULT_TIER_HAWK: u128 = 5_000;
pub const DEFAULT_TIER_EAGLE: u128 = 20_000;
pub const DEFAULT_TIER_PHOENIX: u128 = 60_000;

// default pool
pub const DEFAULT_ADMIN_ID: AccountId = 1;
pub const DEFAULT_POOL_ID: u32 = 1;
pub const DEFAULT_TOKEN: u32 = 1;
pub const DEFAULT_DURATION: u64 = 1000;
pub const DEFAULT_OPEN_TIME: u64 = 1;
pub const DEFAULT_OFFERED_CURRENCY: u32 = 1;
pub const DEFAULT_FUNDING_WALLET: AccountId = 100;

construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		Utility: pallet_utility::{Pallet, Call, Storage, Event},
		Timestamp: pallet_timestamp::{Pallet, Call, Storage},
		Tokens: orml_tokens::{Pallet, Storage, Event<T>, Config<T>},
		Redkite: pallet_redkite::{Pallet, Call, Storage, Event<T>},
	}
);

pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(administrators: Vec<u64>, tiers: Vec<u128>) -> sp_io::TestExternalities {
		let mut storage = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
		// Provide some initial balances
		pallet_balances::GenesisConfig::<Test> {
			balances: vec![(Redkite::account_id(), INIT_BALANCE)],
		}
		.assimilate_storage(&mut storage)
		.unwrap();

		// mock: reward from block 4 to block 14
		pallet_redkite::GenesisConfig::<Test> { administrators, tiers }
			.assimilate_storage(&mut storage)
			.unwrap();

		let mut ext = sp_io::TestExternalities::from(storage);
		ext.execute_with(|| {
			run_to_block(1);
			assert_ok!(Redkite::init_pool(
				Origin::signed(DEFAULT_ADMIN_ID),
				DEFAULT_POOL_ID,
				DEFAULT_TOKEN,
				DEFAULT_DURATION,
				DEFAULT_OPEN_TIME,
				DEFAULT_OFFERED_CURRENCY,
				DEFAULT_FUNDING_WALLET,
			));
		});

		ext
	}
}

pub(crate) fn mock_test() -> sp_io::TestExternalities {
	ExtBuilder::build(
		vec![1u64, 2u64, 3u64],
		vec![
			DEFAULT_TIER_DOVE,
			DEFAULT_TIER_HAWK,
			DEFAULT_TIER_EAGLE,
			DEFAULT_TIER_PHOENIX,
		],
	)
}

// pub(crate) fn events() -> Vec<super::Event<Test>> {
// 	System::events()
// 		.into_iter()
// 		.map(|r| r.event)
// 		.filter_map(|e| {
// 			if let Event::pallet_redkite(inner) = e {
// 				Some(inner)
// 			} else {
// 				None
// 			}
// 		})
// 		.collect::<Vec<_>>()
// }

pub(crate) fn run_to_block(n: u64) {
	while System::block_number() < n {
		System::set_block_number(System::block_number() + 1);
	}
}
