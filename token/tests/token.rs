// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use dusk_core::signatures::bls::{
    PublicKey as AccountPublicKey, SecretKey as AccountSecretKey,
};
use dusk_core::transfer::data::ContractCall;
use dusk_core::transfer::MoonlightTransactionEvent;

use rand::rngs::StdRng;
use rand::SeedableRng;

use tt_core::*;

pub mod instantiate;
use instantiate::{
    TestSession, HOLDER_ID, INITIAL_BALANCE, INITIAL_HOLDER_BALANCE,
    INITIAL_SUPPLY,
};

#[test]
fn deploy() {
    TestSession::new();
}

#[test]
fn empty_account() {
    let mut session = TestSession::new();

    let mut rng = StdRng::seed_from_u64(0xBEEF);
    let sk = AccountSecretKey::random(&mut rng);
    let pk = AccountPublicKey::from(&sk);

    let account = session.account(pk);
    assert_eq!(
        account,
        AccountInfo::EMPTY,
        "An account never transferred to should be empty"
    );
}

// Test that the token contract can not be initialized when it already carries
/// data.
#[test]
fn double_init() {
    const INSERT_VALUE: u64 = INITIAL_BALANCE + 42;

    let mut session = TestSession::new();

    // generate new keys to insert with init functions
    let mut rng = StdRng::seed_from_u64(0xBEEF);
    let sk = AccountSecretKey::random(&mut rng);
    let pk = AccountPublicKey::from(&sk);
    session
        .call_token::<(Vec<(Account, u64)>, Account), ()>(
            &*TestSession::SK_0,
            "init",
            &(
                vec![(Account::External(pk), INSERT_VALUE)],
                Account::External(pk),
            ),
        )
        .expect_err("Call should not pass");

    assert_eq!(
        session.account(pk).balance,
        0,
        "The new account should have 0 balance"
    );

    assert_eq!(
        session.total_supply(),
        INITIAL_SUPPLY,
        "The token supply shouldn't have changed"
    );
}

/// Test a token transfer from the deploy account to the test account.
#[test]
fn transfer() {
    const TRANSFERRED_AMOUNT: u64 = INITIAL_BALANCE - 1;

    let mut session = TestSession::new();

    let receiver_account = Account::from(*TestSession::PK_2);

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE,
        "The deployed account should have the initial balance"
    );

    assert_eq!(
        session.account(receiver_account).balance,
        0,
        "The account to transfer to should have no balance"
    );

    session
        .call_token::<_, ()>(
            &*TestSession::SK_1,
            "transfer",
            &(receiver_account, TRANSFERRED_AMOUNT),
        )
        .expect("Call should pass");

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE - TRANSFERRED_AMOUNT,
        "The deployed account should have the transferred amount subtracted"
    );
    assert_eq!(
        session.account(receiver_account).balance,
        TRANSFERRED_AMOUNT,
        "The account transferred to should have the transferred amount"
    );
}

/// Test a token transfer from the deploy account to the test contract account.
#[test]
fn transfer_to_contract() {
    const TRANSFERRED_AMOUNT: u64 = INITIAL_BALANCE - 1;

    let mut session = TestSession::new();

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE,
        "The deployed account should have the initial balance"
    );
    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE,
        "The receiver contract should have its initial balance"
    );

    let contract_account = Account::from(HOLDER_ID);

    session
        .call_token::<_, ()>(
            &*TestSession::SK_1,
            "transfer",
            &(contract_account, TRANSFERRED_AMOUNT),
        )
        .expect("Call should pass");

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE - TRANSFERRED_AMOUNT,
        "The deployed account should have the transferred amount subtracted"
    );

    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE + TRANSFERRED_AMOUNT,
        "The contract transferred to should have the transferred amount added"
    );

    assert_eq!(
        session.holder_tracked_balance(),
        INITIAL_HOLDER_BALANCE,
        "The contract should have no knowledge of the transfer"
    );
}

/// Test a token transfer and call from the deploy account to the test contract
/// account.
#[test]
fn transfer_and_call_to_contract() {
    const TRANSFERRED_AMOUNT: u64 = INITIAL_BALANCE - 1;

    let mut session = TestSession::new();
    let account_1 = Account::from(*TestSession::PK_1);
    let contract_call = ContractCall::new(
        HOLDER_ID,
        "token_received",
        &(account_1, TRANSFERRED_AMOUNT),
    )
    .expect("Creating contract call should succeed");

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE,
        "The deployed account should have the initial balance"
    );
    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE,
        "The receiver contract should have its initial balance"
    );

    // external transfer

    session
        .call_token::<_, ()>(
            &*TestSession::SK_1,
            "transfer_and_call",
            &(TRANSFERRED_AMOUNT, contract_call),
        )
        .expect("Call should pass");

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE - TRANSFERRED_AMOUNT,
        "The deployed account should have the transferred amount subtracted"
    );

    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE + TRANSFERRED_AMOUNT,
        "The contract transferred to should have the transferred amount added"
    );

    assert_eq!(
        session.holder_tracked_balance(),
        INITIAL_HOLDER_BALANCE + TRANSFERRED_AMOUNT,
        "The contract should have knowledge of the transfer"
    );

    // contract transfer

    // token_send to itself with token_send_and_call
    let contract_call = ContractCall::new(
        HOLDER_ID,
        "token_received",
        &(Account::Contract(HOLDER_ID), TRANSFERRED_AMOUNT),
    )
    .expect("Creating contract call should succeed");

    let receipt = session
        .call_holder::<_, ()>(
            &*TestSession::SK_1,
            "token_send_and_call",
            &(TRANSFERRED_AMOUNT, contract_call),
        )
        .expect("Call should pass");

    receipt.events.iter().for_each(|event| {
        if event.topic == "moonlight" {
            let transfer_info =
                rkyv::from_bytes::<MoonlightTransactionEvent>(&event.data)
                    .unwrap();

            assert!(
                transfer_info.sender == *TestSession::PK_1,
                "The tx origin should be the deploy pk"
            )
        } else if event.topic == "transfer" {
            let transfer_event =
                rkyv::from_bytes::<TransferEvent>(&event.data).unwrap();

            assert!(
                transfer_event.sender == HOLDER_ID.into(),
                "The sender should be the contract"
            );
            assert!(
                transfer_event.receiver == HOLDER_ID.into(),
                "The receiver should be the deploy account"
            );
            assert_eq!(
                transfer_event.value, TRANSFERRED_AMOUNT,
                "The transferred amount should be the same"
            );
        }
    });

    // balance should be the same as before
    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE + TRANSFERRED_AMOUNT,
        "The contract transferred to should have the transferred amount added"
    );

    assert_eq!(
        session.holder_tracked_balance(),
        INITIAL_HOLDER_BALANCE + TRANSFERRED_AMOUNT,
        "The contract should have knowledge of the transfer"
    );
}

/// Test a token transfer from the HOLDER_ID contract account to the deploy
/// account.
#[test]
fn transfer_from_contract() {
    const TRANSFERRED_AMOUNT: u64 = INITIAL_BALANCE - 1;

    let mut session = TestSession::new();
    let account_1 = Account::from(*TestSession::PK_1);

    assert_eq!(
        session.account(account_1).balance,
        INITIAL_BALANCE,
        "The deployed account should have the initial balance"
    );
    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE,
        "The contract to transfer to should have its initial balance"
    );

    let receipt = session
        .call_holder::<_, ()>(
            &*TestSession::SK_1,
            "token_send",
            &(account_1, TRANSFERRED_AMOUNT),
        )
        .expect("Call should pass");

    receipt.events.iter().for_each(|event| {
        if event.topic == "moonlight" {
            let transfer_info =
                rkyv::from_bytes::<MoonlightTransactionEvent>(&event.data)
                    .unwrap();

            assert!(
                transfer_info.sender == *TestSession::PK_1,
                "The tx origin should be the deploy pk"
            )
        } else if event.topic == "transfer" {
            let transfer_event =
                rkyv::from_bytes::<TransferEvent>(&event.data).unwrap();

            assert!(
                transfer_event.sender == HOLDER_ID.into(),
                "The sender should be the contract"
            );
            assert!(
                transfer_event.receiver == (account_1).into(),
                "The receiver should be the deploy account"
            );
            assert_eq!(
                transfer_event.value, TRANSFERRED_AMOUNT,
                "The transferred amount should be the same"
            );
        }
    });

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE + TRANSFERRED_AMOUNT,
        "The deployed account should have the transferred amount added"
    );
    assert_eq!(
        session.account(HOLDER_ID).balance,
        INITIAL_HOLDER_BALANCE - TRANSFERRED_AMOUNT,
        "The contract transferred to should have the transferred amount subtracted"
    );
}

/// Test approval of deploy account to test account.
#[test]
fn approve() {
    const APPROVED_AMOUNT: u64 = INITIAL_BALANCE - 1;

    let mut session = TestSession::new();

    let test_account = Account::from(*TestSession::PK_2);

    assert_eq!(
        session.allowance(*TestSession::PK_1, test_account),
        0,
        "The account should not be allowed to spend tokens from the deployed account"
    );

    session
        .call_token::<_, ()>(
            &*TestSession::SK_1,
            "approve",
            &(test_account, APPROVED_AMOUNT),
        )
        .expect("Call should pass");

    assert_eq!(
        session.allowance(*TestSession::PK_1, test_account),
        APPROVED_AMOUNT,
        "The account should be allowed to spend tokens from the deployed account"
    );
}

/// Test approve from deploy account to test account and
/// transfer from deploy account to test account
/// where sender is deploy account, spender is test account, receiver is test
/// account
#[test]
fn transfer_from() {
    const APPROVED_AMOUNT: u64 = INITIAL_BALANCE - 1;
    const TRANSFERRED_AMOUNT: u64 = APPROVED_AMOUNT / 2;

    let mut session = TestSession::new();
    let spender_account = Account::from(*TestSession::PK_2);
    let owner_account = Account::from(*TestSession::PK_1);

    assert_eq!(
        session.account(owner_account).balance,
        INITIAL_BALANCE,
        "The owner account should have the initial balance"
    );
    assert_eq!(
        session.account(spender_account).balance,
        0,
        "The account to transfer to should have no balance"
    );
    assert_eq!(
        session.allowance(owner_account, spender_account),
        0,
        "The spender account should not be allowed to spend tokens from the owner account"
    );

    session
        .call_token::<_, ()>(
            &*TestSession::SK_1,
            "approve",
            &(spender_account, APPROVED_AMOUNT),
        )
        .expect("Call should pass");

    assert_eq!(
        session.allowance(owner_account, spender_account),
        APPROVED_AMOUNT,
        "The account should be allowed to spend tokens from the deployed account"
    );

    session
        .call_token::<_, ()>(
            &*TestSession::SK_2,
            "transfer_from",
            &(owner_account, spender_account, TRANSFERRED_AMOUNT),
        )
        .expect("Call should pass");

    assert_eq!(
        session.account(*TestSession::PK_1).balance,
        INITIAL_BALANCE - TRANSFERRED_AMOUNT,
        "The deployed account should have the transferred amount subtracted"
    );
    assert_eq!(
        session.account(spender_account).balance,
        TRANSFERRED_AMOUNT,
        "The account transferred to should have the transferred amount"
    );
    assert_eq!(
        session.allowance(*TestSession::PK_1, spender_account),
        APPROVED_AMOUNT - TRANSFERRED_AMOUNT,
        "The account should have the transferred amount subtracted from its allowance"
    );
}
