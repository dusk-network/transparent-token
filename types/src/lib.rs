//! Types used to inteact with the `ttoken-contract`.

#![no_std]
#![deny(missing_docs)]

use bytecheck::CheckBytes;
use rkyv::{Archive, Deserialize, Serialize};

use execution_core::signatures::bls::{PublicKey, SecretKey, Signature};

/// The data an account has in the contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Account {
    /// The balance of the account.
    pub balance: u64,
    /// The current nonce of the account. Use the current value +1 to perform an interaction with
    /// the account.
    pub nonce: u64,
}

impl Account {
    /// An empty account.
    pub const EMPTY: Self = Self {
        balance: 0,
        nonce: 0,
    };
}

/// Arguments to query for how much of an allowance a spender has of the `owner` account.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Allowance {
    /// The account that owns the tokens.
    pub owner: PublicKey,
    /// The account allowed to spend the `owner`s tokens.
    pub spender: PublicKey,
}

/// Data used to transfer tokens from one account to another.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Transfer {
    from: PublicKey,
    to: PublicKey,
    value: u64,
    nonce: u64,
    signature: Signature,
}

impl Transfer {
    const SIGNATURE_MSG_SIZE: usize = 193 + 193 + 8 + 8;

    /// Create a new transfer.
    pub fn new(from_sk: &SecretKey, to: PublicKey, value: u64, nonce: u64) -> Self {
        let from = PublicKey::from(from_sk);

        let mut transfer = Self {
            from,
            to,
            value,
            nonce,
            signature: Signature::default(),
        };

        let sig_msg = transfer.signature_message();
        let sig = from_sk.sign(&sig_msg);
        transfer.signature = sig;

        transfer
    }

    /// The account to transfer from.
    pub fn from(&self) -> &PublicKey {
        &self.from
    }

    /// The account to transfer to.
    pub fn to(&self) -> &PublicKey {
        &self.to
    }

    /// The value to transfer.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The nonce used to sign the transfer.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// The signature used for the transfer.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// The message to be signed over.
    pub fn signature_message(&self) -> [u8; Self::SIGNATURE_MSG_SIZE] {
        let mut msg = [0u8; Self::SIGNATURE_MSG_SIZE];

        let mut offset = 0;

        let bytes = self.from.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.to.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.value.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.nonce.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        // offset += bytes.len();

        msg
    }
}

/// Data used to transfer tokens from an owner to a recipient, by an allowed party.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TransferFrom {
    spender: PublicKey,
    owner: PublicKey,
    to: PublicKey,
    value: u64,
    nonce: u64,
    signature: Signature,
}

impl TransferFrom {
    const SIGNATURE_MSG_SIZE: usize = 193 + 193 + 193 + 8 + 8;

    /// Create a new transfer, spending tokens from the `owner`.
    pub fn new(
        spender_sk: &SecretKey,
        owner: PublicKey,
        to: PublicKey,
        value: u64,
        nonce: u64,
    ) -> Self {
        let spender = PublicKey::from(spender_sk);

        let mut transfer_from = Self {
            spender,
            owner,
            to,
            value,
            nonce,
            signature: Signature::default(),
        };

        let sig_msg = transfer_from.signature_message();
        let sig = spender_sk.sign(&sig_msg);
        transfer_from.signature = sig;

        transfer_from
    }

    /// The account spending the tokens.
    pub fn spender(&self) -> &PublicKey {
        &self.spender
    }

    /// The account that owns the tokens being transferred.
    pub fn owner(&self) -> &PublicKey {
        &self.owner
    }

    /// The account to transfer to.
    pub fn to(&self) -> &PublicKey {
        &self.to
    }

    /// The value to transfer.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The nonce used to sign the transfer.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// The signature used for the transfer.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// The message to be signed over.
    pub fn signature_message(&self) -> [u8; Self::SIGNATURE_MSG_SIZE] {
        let mut msg = [0u8; Self::SIGNATURE_MSG_SIZE];

        let mut offset = 0;

        let bytes = self.spender.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.owner.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.to.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.value.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.nonce.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        // offset += bytes.len();

        msg
    }
}

/// Data used to approve spending tokens from a user's account.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct Approve {
    owner: PublicKey,
    spender: PublicKey,
    value: u64,
    nonce: u64,
    signature: Signature,
}

impl Approve {
    const SIGNATURE_MSG_SIZE: usize = 193 + 193 + 8 + 8;

    /// Create a new approval.
    pub fn new(owner_sk: &SecretKey, spender: PublicKey, value: u64, nonce: u64) -> Self {
        let owner = PublicKey::from(owner_sk);

        let mut approve = Self {
            owner,
            spender,
            value,
            nonce,
            signature: Signature::default(),
        };

        let sig_msg = approve.signature_message();
        let sig = owner_sk.sign(&sig_msg);
        approve.signature = sig;

        approve
    }

    /// The account to allow the transfer of tokens.
    pub fn owner(&self) -> &PublicKey {
        &self.owner
    }

    /// The account to allow spending tokens from.
    pub fn spender(&self) -> &PublicKey {
        &self.spender
    }

    /// The value to approve the transfer of.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// The nonce used to sign the allowance.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// The signature used for the allowance.
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    /// The message to be signed over.
    pub fn signature_message(&self) -> [u8; Self::SIGNATURE_MSG_SIZE] {
        let mut msg = [0u8; Self::SIGNATURE_MSG_SIZE];

        let mut offset = 0;

        let bytes = self.owner.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.spender.to_raw_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.value.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        offset += bytes.len();

        let bytes = self.nonce.to_le_bytes();
        msg[offset..][..bytes.len()].copy_from_slice(&bytes);
        // offset += bytes.len();

        msg
    }
}

/// Event emitted when tokens are transferred from one account to another.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct TransferEvent {
    /// The account tokens are transferred from.
    pub owner: PublicKey,
    /// The account spending the tokens, set if `transfer_from` is used.
    pub spender: Option<PublicKey>,
    /// The account receiving the tokens.
    pub to: PublicKey,
    /// The value transferred.
    pub value: u64,
}

/// Event emitted when a spender is approved on an account.
#[derive(Debug, Clone, Archive, Serialize, Deserialize)]
#[archive_attr(derive(CheckBytes))]
pub struct ApproveEvent {
    /// The account allowing the transfer.
    pub owner: PublicKey,
    /// The allowed spender.
    pub spender: PublicKey,
    /// The value `spender` is allowed to spend.
    pub value: u64,
}
