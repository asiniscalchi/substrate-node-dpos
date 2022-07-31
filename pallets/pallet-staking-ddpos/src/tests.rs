use crate::{mock::*, Error, Config as MyConfig};
use frame_support::{assert_noop, assert_ok};
use frame_support::traits::Currency;


#[test]
fn bond_less_than_minimum_should_fail() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get() - 1;
		assert_noop!(Staking::bond(Origin::signed(ALICE), amount), Error::<Test>::InsufficientBond);
	});
}

#[test]
fn bond_equal_to_minimum_should_be_ok() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get();
		assert_ok!(Staking::bond(Origin::signed(ALICE), amount));
	});
}

#[test]
fn bond_more_than_balance_should_fail() {
	new_test_ext().execute_with(|| {
		let balance = <Test as MyConfig>::Currency::total_balance(&ALICE);
		assert_noop!(Staking::bond(Origin::signed(ALICE), balance + 1), Error::<Test>::InsufficientBond);
	});
}

#[test]
fn bond_all_balance_should_succeed() {
	new_test_ext().execute_with(|| {
		let balance = <Test as MyConfig>::Currency::total_balance(&ALICE);
		assert_ok!(Staking::bond(Origin::signed(ALICE), balance));
	});
}

#[test]
fn bond_twice_should_fail() {
	new_test_ext().execute_with(|| {
		let balance = 100;
		assert_ok!(Staking::bond(Origin::signed(ALICE), balance));
		assert_noop!(Staking::bond(Origin::signed(ALICE), balance), Error::<Test>::AlreadyBonded);
	});
}

