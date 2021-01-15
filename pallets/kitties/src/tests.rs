use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop, traits::{OnInitialize, OnFinalize}};

use super::*;

fn run_to_block( n: u64) {
	while SystemModule::block_number() < n {
		KittyModule::on_finalize(SystemModule::block_number());
		SystemModule::on_finalize(SystemModule::block_number());
		SystemModule::set_block_number(SystemModule::block_number()+1);
		SystemModule::on_initialize(SystemModule::block_number());
		KittyModule::on_initialize(SystemModule::block_number());
	}
}

#[test]
fn kitty_create_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		// Dispatch a signed extrinsic.
		assert_ok!(KittyModule::create(Origin::signed(1)));
		// Read pallet storage and assert an expected result.
		// event 0 is TestEvent::balances(RawEvent::Reserved(1, 1000000))
		assert_eq!(
			SystemModule::events()[1].event,
			TestEvent::kitties(Event::<Test>::Create(1u64, 0))
		);

		assert_eq!(KittiesCount::<Test>::get(), 1);
		assert_eq!(KittiesOwners::<Test>::get(0), Some(1u64));
		assert_eq!(OwnerKitties::<Test>::get(1u64, 0), Some(()));
	})
}

#[test]
fn kitty_create_reserve_error() {
	new_test_ext().execute_with(|| {
		run_to_block(10);
		assert_noop!(
			KittyModule::create(Origin::signed(4)),
			Error::<Test>::ReservedError
		);
	})
}

#[test]
fn kitty_transfer_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));

		assert_ok!(KittyModule::transfer(Origin::signed(1), 2, 0));

		assert_eq!(
			SystemModule::events()[4].event,
			TestEvent::kitties(Event::<Test>::Transfer(1u64, 2u64, 0))
		);

		assert_eq!(KittiesOwners::<Test>::get(0), Some(2u64));
		// origin owner have removed kitty
		assert_eq!(OwnerKitties::<Test>::get(1u64, 0), None);
		// new owner have kitty.
		assert_eq!(OwnerKitties::<Test>::get(2u64, 0), Some(()));
	})
}

#[test]
fn kitty_transfer_invalid_id() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));

		assert_noop!(
			KittyModule::transfer(Origin::signed(1), 2, 1),
			Error::<Test>::InvalidKittyId
		);
	})
}

#[test]
fn kitty_transfer_not_own() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));

		assert_noop!(
			KittyModule::transfer(Origin::signed(3), 2, 0),
			Error::<Test>::KittyNotOwn
		);
	})
}


#[test]
fn kitty_transfer_reserve_error() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));

		assert_noop!(
			KittyModule::transfer(Origin::signed(1), 4, 0),
			Error::<Test>::ReservedError
		);
	})
}


#[test]
fn kitty_breed_works() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));
		KittyModule::create(Origin::signed(1));

		assert_ok!(KittyModule::breed(Origin::signed(1), 0, 1));

		assert_eq!(
			SystemModule::events()[5].event,
			TestEvent::kitties(Event::<Test>::Breed(1u64, 2, 0, 1))
		);

		assert_eq!(KittiesCount::<Test>::get(), 3);
		// new breed kitty owner
		assert_eq!(KittiesOwners::<Test>::get(2), Some(1u64));
		assert_eq!(OwnerKitties::<Test>::get(1u64, 2), Some(()));

		assert_eq!(KittiesParents::<Test>::get(2), Some((0, 1)));

		assert_eq!(KittiesChildren::<Test>::get(0, 2), Some(()));
		assert_eq!(KittiesChildren::<Test>::get(1, 2), Some(()));

		assert_eq!(KittiesBreed::<Test>::get(0, 1), Some(()));
		assert_eq!(KittiesBreed::<Test>::get(1, 0), Some(()));
	})
}


#[test]
fn kitty_breed_reserve_error() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(3));
		KittyModule::create(Origin::signed(3));

		assert_noop!(
			KittyModule::breed(Origin::signed(3), 0, 1),
			Error::<Test>::ReservedError
		);
	})
}

#[test]
fn kitty_breed_different_parent() {
	new_test_ext().execute_with(|| {
		run_to_block(10);

		KittyModule::create(Origin::signed(1));
		KittyModule::create(Origin::signed(1));

		assert_noop!(
			KittyModule::breed(Origin::signed(3), 1, 1),
			Error::<Test>::RequireDifferentParent
		);
	})
}

