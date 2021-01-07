use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

use super::*;

#[test]
fn create_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![1, 16];
        assert_ok!(PoeModule::create_claim(Origin::signed(100), claim.clone()));

        assert_eq!(Proofs::<Test>::get(&claim), (100, frame_system::Module::<Test>::block_number()));
    })
}

#[test]
fn create_claim_failed_when_claim_already_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![1, 16];
        let _ = PoeModule::create_claim(Origin::signed(200), claim.clone());

        assert_noop!(
            PoeModule::create_claim(Origin::signed(200), claim.clone()),
            Error::<Test>::ProofAlreadyExist
        );
    })
}

#[test]
fn revoke_claim_works() {
    new_test_ext().execute_with(|| {
        let claim = vec![1, 16];
        let _ = PoeModule::create_claim(Origin::signed(300), claim.clone());

        assert_ok!(PoeModule::revoke_claim(Origin::signed(300), claim.clone()));
    })
}

#[test]
fn revoke_claim_failed_when_claim_is_not_exist() {
    new_test_ext().execute_with(|| {
        let claim = vec![1, 16];
        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(300), claim.clone()),
            Error::<Test>::ClaimNotExist
        );
    })
}

#[test]
fn revoke_claim_failed_when_not_owner() {
    new_test_ext().execute_with(|| {
        let claim = vec![1, 16];
        let _ = PoeModule::create_claim(Origin::signed(100), claim.clone());
        assert_noop!(
            PoeModule::revoke_claim(Origin::signed(300), claim.clone()),
            Error::<Test>::NotClaimOwner
        );
    })
}

