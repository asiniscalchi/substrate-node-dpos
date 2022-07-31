use crate::{mock::*, Error, Config as MyConfig};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::Currency;

#[test]
fn bond_less_than_minimum_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get() - 1;
		assert_noop!(Staking::bond(Origin::signed(1), amount), Error::<Test>::InsufficientBond);
	});
}

#[test]
fn bond_equal_to_minimum_should_be_ok() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get();
		assert_ok!(Staking::bond(Origin::signed(1), amount));
	});
}

#[test]
fn bond_more_than_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id = 1;
		let balance = <Test as MyConfig>::Currency::total_balance(&account_id);
		assert_noop!(Staking::bond(Origin::signed(account_id), balance + 1), Error::<Test>::InsufficientBond);
	});
}

#[test]
fn bond_all_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let account_id = 1;
		let balance = <Test as MyConfig>::Currency::total_balance(&account_id);
		assert_noop!(Staking::bond(Origin::signed(account_id), balance), Error::<Test>::InsufficientBond);
	});
}
