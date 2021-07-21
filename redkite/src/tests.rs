use crate::*;
use frame_support::{assert_noop, assert_ok};
use mock::*;

#[test]
fn init_pool_work() {
	mock_test().execute_with(|| {
		assert_eq!(Redkite::pot(), INIT_BALANCE - MINIMUM_BALANCE);

		// test administrator permission
		assert_noop!(
			Redkite::init_pool(
				Origin::signed(4),
				DEFAULT_POOL_ID,
				DEFAULT_TOKEN,
				DEFAULT_DURATION,
				DEFAULT_OPEN_TIME,
				DEFAULT_OFFERED_CURRENCY,
				DEFAULT_FUNDING_WALLET,
			),
			Error::<Test>::InvalidPermission
		);

		assert_eq!(Redkite::pools(DEFAULT_POOL_ID	).unwrap().funding_wallet, DEFAULT_FUNDING_WALLET);
	})
}

#[test]
fn set_open_time_work() {
	mock_test().execute_with(|| {
		const NEW_OPEN_TIME: u64 = 64;
		// test administrator permission
		assert_noop!(
			Redkite::set_open_time(Origin::signed(4), DEFAULT_POOL_ID, NEW_OPEN_TIME),
			Error::<Test>::InvalidPermission
		);
		assert_ok!(Redkite::set_open_time(Origin::signed(DEFAULT_ADMIN_ID), DEFAULT_POOL_ID, NEW_OPEN_TIME));
		assert_eq!(Redkite::pools(DEFAULT_POOL_ID).unwrap().open_time, NEW_OPEN_TIME);
	})
}

#[test]
fn set_close_time_work() {
	mock_test().execute_with(|| {
		const NEW_CLOSE_TIME: u64 = 64;
		// test administrator permission
		assert_noop!(
			Redkite::set_close_time(Origin::signed(4), DEFAULT_POOL_ID, NEW_CLOSE_TIME),
			Error::<Test>::InvalidPermission
		);
		assert_ok!(Redkite::set_close_time(Origin::signed(DEFAULT_ADMIN_ID), DEFAULT_POOL_ID, NEW_CLOSE_TIME));
		assert_eq!(Redkite::pools(DEFAULT_POOL_ID).unwrap().close_time, NEW_CLOSE_TIME);
	})
}

#[test]
fn grant_administrators_work() {
	mock_test().execute_with(|| {
		const NEW_CLOSE_TIME: u64 = 64;
		const NEW_ADMIN_ID: u64 = 10;
		// test administrator permission
		assert_noop!(
			Redkite::set_close_time(Origin::signed(NEW_ADMIN_ID), DEFAULT_POOL_ID, NEW_CLOSE_TIME),
			Error::<Test>::InvalidPermission
		);
		assert_ok!(Redkite::grant_administrators(Origin::signed(DEFAULT_ADMIN_ID), vec![NEW_ADMIN_ID]));
		assert_ok!(Redkite::set_close_time(Origin::signed(NEW_ADMIN_ID), DEFAULT_POOL_ID, NEW_CLOSE_TIME));
		assert_eq!(Redkite::pools(DEFAULT_POOL_ID).unwrap().close_time, NEW_CLOSE_TIME);
	})
}

#[test]
fn set_pool_winners_work() {
	mock_test().execute_with(|| {
		assert_eq!(Redkite::pot(), INIT_BALANCE - MINIMUM_BALANCE);
		assert_eq!(Redkite::pools(1).unwrap().funding_wallet, DEFAULT_FUNDING_WALLET);
	})
}
