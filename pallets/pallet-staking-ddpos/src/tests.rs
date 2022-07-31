use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok};

#[test]
fn bond_less_than_minimum_should_raise_an_error() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get() - 1;
		assert_noop!(Staking::bond(Origin::signed(1000), amount), Error::<Test>::InsufficientBond);
	});
}

#[test]
fn bond_equal_to_minimum_should_be_ok() {
	new_test_ext().execute_with(|| {
		let amount = ExistentialDeposit::get();
		assert_ok!(Staking::bond(Origin::signed(1000), amount));
	});
}
