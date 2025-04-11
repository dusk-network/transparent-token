// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
// Copyright (c) DUSK NETWORK. All rights reserved.

use core::cmp::Ordering;

use bytecheck::CheckBytes;
use dusk_core::abi::ContractId;
use dusk_core::signatures::bls::PublicKey;
use rkyv::{Archive, Deserialize, Serialize};

/// Error messages for when an account doesn't have enough tokens to perform the
/// desired operation.
pub const BALANCE_TOO_LOW: &str = "The account doesn't have enough tokens";

/// Error message for when the account is not found in the contract.
pub const ACCOUNT_NOT_FOUND: &str = "The account does not exist";

/// Error message for when a wrong contract calls the contract.
pub const INVALID_CALLER: &str = "Invalid caller";

/// Shielded transactions are not supported.
pub const SHIELDED_NOT_SUPPORTED: &str =
    "Shielded transactions are not supported";

/// The label for an account.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[archive_attr(derive(CheckBytes))]
pub enum Account {
    /// An externally owned account.
    External(PublicKey),
    /// A contract account.
    Contract(ContractId),
}

impl From<PublicKey> for Account {
    fn from(pk: PublicKey) -> Self {
        Self::External(pk)
    }
}

impl From<ContractId> for Account {
    fn from(contract: ContractId) -> Self {
        Self::Contract(contract)
    }
}

// The implementations of `PartialOrd` and `Ord`, while technically meaningless,
// are extremely useful for using `Account` as keys of a `BTreeMap` in the
// contract.

impl PartialOrd for Account {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Account {
    fn cmp(&self, other: &Self) -> Ordering {
        use Account::{Contract, External};

        match (self, other) {
            (External(lhs), External(rhs)) => {
                let lhs = lhs.to_raw_bytes();
                let rhs = rhs.to_raw_bytes();
                lhs.cmp(&rhs)
            }
            (Contract(lhs), Contract(rhs)) => lhs.cmp(rhs),
            // An externally owned account is defined as always "smaller" than a
            // contract account. This ensures they are never mixed
            // when ordering.
            (External(_lhs), Contract(_rhs)) => Ordering::Greater,
            (Contract(_lhs), External(_rhs)) => Ordering::Less,
        }
    }
}

/// The data an account has in the contract.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize,
)]
#[archive_attr(derive(CheckBytes))]
#[allow(clippy::module_name_repetitions)]
pub struct AccountInfo {
    /// The balance of the account.
    pub balance: u64,
}

impl AccountInfo {
    /// An empty account.
    pub const EMPTY: Self = Self { balance: 0 };
}
