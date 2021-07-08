use super::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn elect_feeder_should_works() {
	ExtBuilder::default()
		.alice_is_feeder()
		.build_and_execute(|| {
			assert_noop!(
				Oracle::elect_feeder(
					Origin::root(),
					ALICE,
				),
				Error::<Test, _>::AlreadyFeeder
			);
			assert_ok!(
				Oracle::elect_feeder(
					Origin::root(),
					BOB,
				),
			);
			assert_eq!(Feeders::<Test, _>::get().len(), 2)
		})
}

#[test]
fn set_fee_should_work() {
	ExtBuilder::default()
		.alice_is_feeder()
		.build_and_execute(|| {
			assert_noop!(
				Oracle::set_fee(
					Origin::signed(POLKAFOUNDRY),
					100u64,
				),
				Error::<Test, _>::NoPermission
			);
			assert_ok!(
				Oracle::set_fee(
					Origin::signed(ALICE),
					100u64,
				),
			);
		})
}

#[test]
fn feed_value_should_work() {
	ExtBuilder::default()
		.alice_is_feeder()
		.build_and_execute(|| {
			assert_noop!(
				Oracle::feed_values(
					Origin::signed(BOB),
					vec![(1, 2)]
				),
				Error::<Test, _>::NoPermission
			);
			assert_eq!(
				Oracle::feed_values(Origin::signed(ALICE), vec![(1, 2)])
					.unwrap()
					.pays_fee,
				Pays::No
			);
			assert_noop!(
				Oracle::feed_values(
					Origin::signed(ALICE),
					vec![(1, 2)]
				),
				Error::<Test, _>::AlreadyFeeded
			);
			Oracle::on_finalize(1);
			assert_ok!(
				Oracle::feed_values(
					Origin::signed(ALICE),
					vec![(1, 2)]
				),
			);
			assert_eq!(
				Oracle::all_value(ALICE, 1),
				Some(TimestampedValue {
					value: 2,
					timestamp: 12345
				})
			);
			Oracle::on_finalize(2);
			assert_ok!(
				Oracle::feed_values(
					Origin::signed(ALICE),
					vec![(1, 5)]
				),
			);
			assert_eq!(
				Oracle::all_value(ALICE, 1),
				Some(TimestampedValue {
					value: 5,
					timestamp: 12345
				})
			);
		})
}

#[test]
fn combine_should_work() {
	ExtBuilder::default()
		.feeders(
			vec![POLKAFOUNDRY, ALICE, BOB]
		)
		.build_and_execute(|| {
			let key: u32 = 50;

			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 1300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 1000)]));
			// not enough feed
			assert_eq!(Oracle::get(&key), None);

			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(key, 1200)]));

			let expected = Some(TimestampedValue {
				value: 1200,
				timestamp: 12345,
			});

			assert_eq!(Oracle::get(&key), expected);

			Timestamp::set_timestamp(23456);

			assert_eq!(Oracle::get(&key), expected);

			Oracle::on_finalize(1);

			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 2300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 2000)]));

			assert_eq!(Oracle::get(&key), expected);

			Oracle::on_finalize(2);

			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 2300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 2000)]));
			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(key, 2200)]));
			let expected2 = Some(TimestampedValue {
				value: 2200,
				timestamp: 23456,
			});
			assert_eq!(Oracle::get(&key), expected2);
			Timestamp::set_timestamp(34567);
			Oracle::on_finalize(2);
			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(key, 2200)]));

			Timestamp::set_timestamp(40000);
			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 2300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 2000)]));
			// still old values because bob is expired
			let expected3 = Some(TimestampedValue {
				value: 2200,
				timestamp: 23456,
			});
			assert_eq!(Oracle::get(&key), expected3);
		})
}

#[test]
fn get_concrete_should_work() {
	ExtBuilder::default()
		.feeders(
			vec![POLKAFOUNDRY, ALICE, BOB]
		)
		.build_and_execute(|| {
			let key: u32 = 50;

			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 1300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 1000)]));
			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(key, 1200)]));
			assert_eq!(
				Oracle::get_concrete(&key, POLKAFOUNDRY),
				Some(TimestampedValue {
					value: 1300,
					timestamp: 12345
				})
			);
			assert_eq!(
				Oracle::get_concrete(&key, ALICE),
				Some(TimestampedValue {
					value: 1000,
					timestamp: 12345
				})
			);
			assert_eq!(
				Oracle::get_concrete(&key, BOB),
				Some(TimestampedValue {
					value: 1200,
					timestamp: 12345
				})
			);
		})
}

#[test]
fn get_all_should_work() {
	ExtBuilder::default()
		.feeders(
			vec![POLKAFOUNDRY, ALICE, BOB]
		)
		.build_and_execute(|| {
			let dot: u32 = 50;
			let ksm: u32 = 60;
			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(dot, 1300), (ksm, 10000)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(dot, 1000),  (ksm, 11000),  (ksm, 12000)]));
			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(dot, 1200), (dot, 1300), (ksm, 13000)]));

			// not combined yet
			assert_eq!(
				Oracle::get_all_values(),
				vec![]
			);
			assert_eq!(Oracle::get(&dot), Some(TimestampedValue {
				value: 1300,
				timestamp: 12345,
			}));
			let dot_price = Some(TimestampedValue {
				value: 1300,
				timestamp: 12345,
			});

			assert_eq!(Oracle::get_all_values(), vec![(dot, dot_price)]);

			let ksm_price = Some(TimestampedValue {
				value: 12000,
				timestamp: 12345,
			});

			assert_eq!(Oracle::get(&ksm), ksm_price);

			assert_eq!(Oracle::get_all_values(), vec![(ksm, ksm_price), (dot, dot_price)]);
		})
}

#[test]
fn remove_feeder_work() {
	ExtBuilder::default()
		.feeders(
			vec![POLKAFOUNDRY, ALICE, BOB]
		)
		.build_and_execute(|| {
			assert_noop!(Oracle::remove_feeder(Origin::root(), 10u64), Error::<Test, _>::NotFeeder);
			assert_noop!(
				Oracle::elect_feeder(
					Origin::root(),
					BOB,
				),
				Error::<Test, _>::AlreadyFeeder
			);

			let key: u32 = 50;
			assert_ok!(Oracle::feed_values(Origin::signed(POLKAFOUNDRY), vec![(key, 1300)]));
			assert_ok!(Oracle::feed_values(Origin::signed(ALICE), vec![(key, 1000)]));
			assert_ok!(Oracle::feed_values(Origin::signed(BOB), vec![(key, 1200)]));
			assert_eq!(
				Oracle::get_concrete(&key, BOB),
				Some(TimestampedValue {
					value: 1200,
					timestamp: 12345
				})
			);
			assert_ok!(Oracle::remove_feeder(Origin::root(), BOB));
			assert_eq!(
				Oracle::get_concrete(&key, BOB),
				None
			);
			assert_ok!(
				Oracle::elect_feeder(
					Origin::root(),
					BOB,
				),
			);
		})
}
