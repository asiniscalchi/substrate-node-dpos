use crate::{
	mock::*, Config as MyConfig, Error, Event, MaximumValidatorCount, MinimumValidatorCount,
};
use frame_support::{assert_noop, assert_ok};
use pallet_session::SessionManager;
use sp_runtime::DispatchError;

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

#[test]
fn bond_unbond_events() {
	new_test_ext().execute_with(|| {
		System::set_block_number(1);

		assert_ok!(Staking::bond(Origin::signed(ALICE), 10));
		assert_eq!(staking_events(), vec![Event::Bonded(ALICE, 10)]);
		assert_ok!(Staking::unbond(Origin::signed(ALICE)));
		assert_eq!(staking_events(), vec![Event::Bonded(ALICE, 10), Event::Unbonded(ALICE),]);
	});
}

#[test]
fn new_session_with_no_validators_should_return_none() {
	new_test_ext().execute_with(|| {
		assert!(Staking::new_session(0).is_none());
	});
}

#[test]
fn new_session_with_validators_should_return_validators() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::new_session(0), None);
		assert_ok!(Staking::bond(Origin::signed(ALICE), 10));
		assert_eq!(Staking::new_session(0), Some(vec![ALICE]));
		assert_ok!(Staking::bond(Origin::signed(BOB), 10));
		assert_eq!(Staking::new_session(0), Some(vec![ALICE, BOB]));
	});
}

#[test]
fn minimum_validator_count_default() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::minimum_validator_count(), MinimumValidatorCount::<Test>::get());
	});
}

#[test]
fn maximum_validator_count_default() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::maximum_validator_count(), MaximumValidatorCount::<Test>::get());
	});
}

#[test]
fn set_minimum_validator_should_be_called_by_root() {
	new_test_ext().execute_with(|| {
		let counter = Staking::minimum_validator_count();
		assert_noop!(
			Staking::set_minimum_validator_count(Origin::signed(ALICE), counter + 1),
			DispatchError::BadOrigin
		);
		assert_ok!(Staking::set_minimum_validator_count(Origin::root(), counter + 1));
		assert_eq!(Staking::minimum_validator_count(), counter + 1);
	});
}

#[test]
fn set_minimum_should_not_0_and_major_than_maximum() {
	new_test_ext().execute_with(|| {
		assert_noop!(
			Staking::set_minimum_validator_count(Origin::root(), 0),
			Error::<Test>::InvalidNumberOfValidators
		);
		let counter = Staking::maximum_validator_count();
		assert_noop!(
			Staking::set_minimum_validator_count(Origin::root(), counter + 1),
			Error::<Test>::InvalidNumberOfValidators
		);
		assert_ok!(Staking::set_minimum_validator_count(Origin::root(), counter));
	});
}

#[test]
fn set_maximum_validator_should_be_called_by_root() {
	new_test_ext().execute_with(|| {
		let counter = Staking::maximum_validator_count();
		assert_noop!(
			Staking::set_maximum_validator_count(Origin::signed(ALICE), counter + 1),
			DispatchError::BadOrigin
		);
		assert_ok!(Staking::set_maximum_validator_count(Origin::root(), counter + 1));
		assert_eq!(Staking::maximum_validator_count(), counter + 1);
	});
}

#[test]
fn set_maximum_should_not_be_minor_than_minimum() {
	new_test_ext().execute_with(|| {
		let counter = Staking::minimum_validator_count();
		assert_noop!(
			Staking::set_maximum_validator_count(Origin::root(), counter - 1),
			Error::<Test>::InvalidNumberOfValidators
		);
		assert_ok!(Staking::set_maximum_validator_count(Origin::root(), counter));
	});
}

#[test]
fn new_session_should_return_at_maximum_maximum_validator() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::maximum_validator_count(), 2);
		assert_ok!(Staking::bond(Origin::signed(1), 10));
		assert_ok!(Staking::bond(Origin::signed(2), 10));
		assert_ok!(Staking::bond(Origin::signed(3), 10));
		let validators = Staking::new_session(1).unwrap();
		assert_eq!(validators.len(), 2);
	});
}

#[test]
fn new_session_should_return_the_winners() {
	new_test_ext().execute_with(|| {
		assert_eq!(Staking::maximum_validator_count(), 2);
		assert_ok!(Staking::bond(Origin::signed(BOB), 20));
		assert_ok!(Staking::bond(Origin::signed(ALICE), 50));
		assert_ok!(Staking::bond(Origin::signed(CHARLIE), 90));
		let validators = Staking::new_session(1);
		assert_eq!(validators, Some(vec![CHARLIE, ALICE]));
	});
}

#[test]
fn users_should_vote_once() {
	new_test_ext().execute_with(|| {
		assert_ok!(Staking::vote(Origin::signed(ALICE), BOB));
		assert_noop!(Staking::vote(Origin::signed(ALICE), CHARLIE), Error::<Test>::AlreadyVoted);
	});
}

#[test]
fn user_should_be_able_to_unvote_always() {
	new_test_ext().execute_with(|| {
		assert_ok!(Staking::unvote(Origin::signed(ALICE)));
		assert_ok!(Staking::unvote(Origin::signed(ALICE)));
		assert_ok!(Staking::unvote(Origin::signed(ALICE)));
	});
}

#[test]
fn users_could_revote_after_unvote() {
	new_test_ext().execute_with(|| {
		assert_ok!(Staking::vote(Origin::signed(ALICE), BOB));
		assert_ok!(Staking::unvote(Origin::signed(ALICE)));
		assert_ok!(Staking::vote(Origin::signed(ALICE), CHARLIE));
	});
}
