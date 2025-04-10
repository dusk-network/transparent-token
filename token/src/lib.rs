// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

#![cfg(target_family = "wasm")]
#![deny(rustdoc::broken_intra_doc_links)]
#![deny(clippy::pedantic)]
#![deny(unused_crate_dependencies)]
#![deny(unused_extern_crates)]
#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

use dusk_core::abi;
use dusk_core::transfer::data::ContractCall;
use tt_core::{
    Account, AccountInfo, ApproveEvent, TransferEvent, ACCOUNT_NOT_FOUND,
    BALANCE_TOO_LOW, SHIELDED_NOT_SUPPORTED, ZERO_ADDRESS,
};

/// The state of the token-contract.
struct TokenState {
    accounts: BTreeMap<Account, AccountInfo>,
    allowances: BTreeMap<Account, BTreeMap<Account, u64>>,
    supply: u64,
}

impl TokenState {
    fn init(&mut self, accounts: Vec<(Account, u64)>) {
        for (account, balance) in accounts {
            let account_entry =
                self.accounts.entry(account).or_insert(AccountInfo::EMPTY);
            account_entry.balance += balance;
            self.supply += balance;

            abi::emit(
                "mint",
                TransferEvent {
                    sender: ZERO_ADDRESS,
                    spender: None,
                    receiver: account,
                    value: balance,
                },
            );
        }
    }
}

static mut STATE: TokenState = TokenState {
    accounts: BTreeMap::new(),
    allowances: BTreeMap::new(),
    supply: 0,
};

/// Basic token-contract implementation.
impl TokenState {
    fn name() -> String {
        String::from("Transparent Fungible Token Sample")
    }

    fn symbol() -> String {
        String::from("TFTS")
    }

    fn decimals() -> u8 {
        18
    }

    fn total_supply(&self) -> u64 {
        self.supply
    }

    fn account(&self, account: Account) -> AccountInfo {
        self.accounts
            .get(&account)
            .copied()
            .unwrap_or(AccountInfo::EMPTY)
    }

    fn balance_of(&self, account: Account) -> u64 {
        self.accounts
            .get(&account)
            .map_or(0, |account| account.balance)
    }

    #[allow(clippy::large_types_passed_by_value)]
    fn allowance(&self, owner: Account, spender: Account) -> u64 {
        match self.allowances.get(&owner) {
            Some(allowances) => allowances.get(&spender).copied().unwrap_or(0),
            None => 0,
        }
    }

    /// Initates a `Transfer` from the sender to the receiver with the specified
    /// value.
    ///
    /// Both the sender and the receiver are accounts.
    ///
    /// # Note
    /// the sender must not be blocked or frozen.
    /// the receiver must not be blocked but can be frozen.
    #[allow(clippy::large_types_passed_by_value)]
    fn transfer(&mut self, receiver: Account, value: u64) {
        let sender = sender_account();

        let sender_account =
            self.accounts.get_mut(&sender).expect(ACCOUNT_NOT_FOUND);

        assert!(sender_account.balance >= value, "{}", BALANCE_TOO_LOW);

        sender_account.balance -= value;

        let receiver_account =
            self.accounts.entry(receiver).or_insert(AccountInfo::EMPTY);

        // this can never overflow as value + balance is never higher than total
        // supply
        receiver_account.balance += value;

        abi::emit(
            TransferEvent::TRANSFER_TOPIC,
            TransferEvent {
                sender,
                spender: None,
                receiver,
                value,
            },
        );
    }

    /// Transfers tokens to a contract receiver and call a specified function on
    /// that contract.
    ///
    /// # Behavior
    ///
    /// This function transfers the given `value` of tokens to the contract
    /// indicated by `contract_call.contract` and then calls the function
    /// specified by `contract_call.fn_name` with the provided
    /// `contract_call.fn_args`.
    ///
    /// If the contract function expects parameters, it is possible to pass
    /// incorrect arguments intentionally. The receiving contract is
    /// responsible for validating such arguments, as this token is unaware
    /// of arbitrary contract logic.
    ///
    ///
    /// # Notes
    ///
    /// - This function cannot be used if you need to transfer tokens to an
    ///   arbitrary account while calling a function on a contract. For
    ///   scenarios requiring multiple operations at once, consider implementing
    ///   a multicall solution.
    /// - `transfer_and_call` is atomic: if the function call on the receiving
    ///   contract fails (due to a panic or out of gas error), the token
    ///   transfer also fails and reverts.
    fn transfer_and_call(&mut self, value: u64, contract_call: &ContractCall) {
        let receiver = Account::from(contract_call.contract);
        self.transfer(receiver, value);

        // If the call to the contract fails (panic or OoG) the transfer
        // also fails.
        if let Err(err) = abi::call_raw(
            contract_call.contract,
            &contract_call.fn_name,
            &contract_call.fn_args,
        ) {
            panic!(
                "Failed calling `{}` on the contract: {err}",
                contract_call.fn_name
            );
        }
    }

    /// Note:
    /// the spender must not be blocked or frozen.
    /// the actual owner of the funds must not be blocked or frozen.
    /// the receiver must not be blocked but can be frozen.
    #[allow(clippy::large_types_passed_by_value)]
    fn transfer_from(&mut self, owner: Account, receiver: Account, value: u64) {
        let spender = sender_account();

        let allowance = self
            .allowances
            .get_mut(&owner)
            .expect("The account has no allowances")
            .get_mut(&spender)
            .expect("The spender is not allowed to use the account");

        assert!(
            value <= *allowance,
            "The spender can't spent the defined amount"
        );

        let owner_account =
            self.accounts.get_mut(&owner).expect(ACCOUNT_NOT_FOUND);

        assert!(owner_account.balance >= value, "{}", BALANCE_TOO_LOW);

        *allowance -= value;
        owner_account.balance -= value;

        let receiver_account =
            self.accounts.entry(receiver).or_insert(AccountInfo::EMPTY);

        // this can never overflow as value + balance is never higher than total
        // supply
        receiver_account.balance += value;

        abi::emit(
            TransferEvent::TRANSFER_TOPIC,
            TransferEvent {
                sender: owner,
                spender: Some(spender),
                receiver,
                value,
            },
        );
    }

    fn approve(&mut self, spender: Account, value: u64) {
        // owner of the funds
        let owner = sender_account();

        let allowances = self.allowances.entry(owner).or_default();

        allowances.insert(spender, value);

        abi::emit(
            ApproveEvent::APPROVE_TOPIC,
            ApproveEvent {
                sender: owner,
                spender,
                value,
            },
        );
    }
}

#[no_mangle]
unsafe extern "C" fn init(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |initial_accounts| {
        STATE.init(initial_accounts);
    })
}

#[no_mangle]
unsafe extern "C" fn name(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| TokenState::name())
}

#[no_mangle]
unsafe extern "C" fn symbol(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| TokenState::symbol())
}

#[no_mangle]
unsafe extern "C" fn decimals(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| TokenState::decimals())
}

#[no_mangle]
unsafe extern "C" fn total_supply(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(): ()| STATE.total_supply())
}

#[no_mangle]
unsafe extern "C" fn account(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |arg| STATE.account(arg))
}

#[no_mangle]
unsafe extern "C" fn balance_of(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |account| STATE.balance_of(account))
}

#[no_mangle]
unsafe extern "C" fn allowance(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(owner, spender)| STATE.allowance(owner, spender))
}

#[no_mangle]
unsafe extern "C" fn transfer(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(receiver, value)| STATE.transfer(receiver, value))
}

#[no_mangle]
unsafe extern "C" fn transfer_and_call(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(transfer, contract_call)| {
        STATE.transfer_and_call(transfer, &contract_call);
    })
}

#[no_mangle]
unsafe extern "C" fn transfer_from(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(owner, receiver, value)| {
        STATE.transfer_from(owner, receiver, value);
    })
}

#[no_mangle]
unsafe extern "C" fn approve(arg_len: u32) -> u32 {
    abi::wrap_call(arg_len, |(spender, value)| STATE.approve(spender, value))
}

/*
 * Helper functions
 */

/// Determines and returns the sender of the current transfer.
///
/// If the sender is an external account, return the transaction origin.
/// If the sender is a contract, return the calling contract.
///
/// # Returns
///
/// An `Account` representing the token sender.
///
/// # Panics
///
/// - If no public sender is available (shielded transactions are not supported)
/// - If no caller can be determined (impossible case)
fn sender_account() -> Account {
    let tx_origin = abi::public_sender().expect(SHIELDED_NOT_SUPPORTED);

    let caller = abi::caller().expect("ICC expects a caller");

    // Identifies the sender by checking the call stack and transaction origin:
    // - For direct external account transactions (call stack length = 1),
    //   returns the transaction origin
    // - For non-protocol contracts that call the token (call stack length > 1),
    //   returns the immediate calling contract
    if abi::callstack().len() == 1 {
        // This also implies, that the call directly originates via the protocol
        // transfer contract i.e., the caller is the transfer
        // contract
        Account::External(tx_origin)
    } else {
        Account::Contract(caller)
    }
}
