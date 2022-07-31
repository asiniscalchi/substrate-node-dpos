use crate::{mock::*, Config as MyConfig, Error};
use frame_support::{assert_noop, assert_ok};

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
fn bond_all_balance_should_succeed() {
	new_test_ext().execute_with(|| {
		let free_balance = <Test as MyConfig>::Currency::free_balance(&ALICE);
		assert_ok!(Staking::bond(Origin::signed(ALICE), free_balance));
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

#[test]
fn unbond_the_unbounded_should_fail() {
	new_test_ext().execute_with(|| {
		assert_noop!(Staking::unbond(Origin::signed(ALICE)), Error::<Test>::NotStash);
	});
}

#[test]
fn bond_unbond_bond_should_succeed() {
	new_test_ext().execute_with(|| {
		assert_ok!(Staking::bond(Origin::signed(ALICE), 10));
		assert_ok!(Staking::unbond(Origin::signed(ALICE)));
		assert_ok!(Staking::bond(Origin::signed(ALICE), 10));
	});
}
