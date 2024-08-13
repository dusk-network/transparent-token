#![no_std]

extern crate alloc;

use alloc::collections::BTreeMap;
use alloc::string::String;

use execution_core::signatures::bls::PublicKey;

use ttoken_types::*;

struct TokenState {
    accounts: BTreeMap<[u8; 193], Account>,
    allowances: BTreeMap<[u8; 193], BTreeMap<[u8; 193], u64>>,
    supply: u64,
}

static mut STATE: TokenState = TokenState {
    accounts: BTreeMap::new(),
    allowances: BTreeMap::new(),
    supply: 0,
};

impl TokenState {
    fn name(&self) -> String {
        String::from("Transparent Fungible Token Sample")
    }

    fn symbol(&self) -> String {
        String::from("TFTS")
    }

    fn decimals(&self) -> u8 {
        18
    }

    fn total_supply(&self) -> u64 {
        self.supply
    }

    fn account(&self, pk: PublicKey) -> Account {
        let account_bytes = pk.to_raw_bytes();

        self.accounts
            .get(&account_bytes)
            .copied()
            .unwrap_or(Account::EMPTY)
    }

    fn allowance(&self, allowance: Allowance) -> u64 {
        let owner_bytes = allowance.owner.to_raw_bytes();
        match self.allowances.get(&owner_bytes) {
            Some(allowances) => {
                let spender_bytes = allowance.spender.to_raw_bytes();
                allowances.get(&spender_bytes).copied().unwrap_or(0)
            }
            None => 0,
        }
    }

    fn transfer(&mut self, transfer: Transfer) {
        let from = *transfer.from();
        let from_bytes = from.to_raw_bytes();

        let from_account = self
            .accounts
            .get_mut(&from_bytes)
            .expect("The account has no tokens to transfer");

        let value = transfer.value();
        if from_account.balance < value {
            panic!("The account doesn't have enough tokens");
        }

        if transfer.nonce() != from_account.nonce + 1 {
            panic!("Nonces must be sequential");
        }

        from_account.balance -= value;
        from_account.nonce += 1;

        let sig = *transfer.signature();
        let sig_msg = transfer.signature_message().to_vec();
        if !rusk_abi::verify_bls(sig_msg, from, sig) {
            panic!("Invalid signature");
        }

        let to = *transfer.to();
        let to_bytes = to.to_raw_bytes();
        let to_account = self.accounts.entry(to_bytes).or_insert(Account::EMPTY);

        to_account.balance += value;

        rusk_abi::emit(
            "transfer",
            TransferEvent {
                owner: from,
                spender: None,
                to,
                value,
            },
        );
    }

    fn transfer_from(&mut self, transfer_from: TransferFrom) {
        let spender = *transfer_from.spender();
        let spender_bytes = spender.to_raw_bytes();

        let spender_account = self.accounts.entry(spender_bytes).or_insert(Account::EMPTY);
        if transfer_from.nonce() != spender_account.nonce + 1 {
            panic!("Nonces must be sequential");
        }

        spender_account.nonce += 1;

        let sig = *transfer_from.signature();
        let sig_msg = transfer_from.signature_message().to_vec();
        if !rusk_abi::verify_bls(sig_msg, spender, sig) {
            panic!("Invalid signature");
        }

        let owner = *transfer_from.owner();
        let owner_bytes = owner.to_raw_bytes();

        let allowance = self
            .allowances
            .get_mut(&owner_bytes)
            .expect("The account has no allowances")
            .get_mut(&spender_bytes)
            .expect("The spender is not allowed to use the account");

        let value = transfer_from.value();
        if value > *allowance {
            panic!("The spender can't spent the defined amount");
        }

        let owner_account = self
            .accounts
            .get_mut(&owner_bytes)
            .expect("The account has no tokens to transfer");

        if owner_account.balance < value {
            panic!("The account doesn't have enough tokens");
        }

        *allowance -= value;
        owner_account.balance -= value;

        let to = *transfer_from.to();
        let to_bytes = to.to_raw_bytes();
        let to_account = self.accounts.entry(to_bytes).or_insert(Account::EMPTY);

        to_account.balance += value;

        rusk_abi::emit(
            "transfer",
            TransferEvent {
                owner,
                spender: Some(spender),
                to,
                value,
            },
        );
    }

    fn approve(&mut self, approve: Approve) {
        let owner = *approve.owner();
        let owner_bytes = owner.to_raw_bytes();

        let owner_account = self.accounts.entry(owner_bytes).or_insert(Account::EMPTY);
        if approve.nonce() != owner_account.nonce + 1 {
            panic!("Nonces must be sequential");
        }

        owner_account.nonce += 1;

        let sig = *approve.signature();
        let sig_msg = approve.signature_message().to_vec();
        if !rusk_abi::verify_bls(sig_msg, owner, sig) {
            panic!("Invalid signature");
        }

        let spender = *approve.spender();
        let spender_bytes = spender.to_raw_bytes();

        let allowances = self
            .allowances
            .entry(owner_bytes)
            .or_insert(BTreeMap::new());

        let value = approve.value();
        allowances.insert(spender_bytes, value);

        rusk_abi::emit(
            "approve",
            ApproveEvent {
                owner,
                spender,
                value,
            },
        );
    }
}

#[no_mangle]
unsafe fn name(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |_: ()| STATE.name())
}

#[no_mangle]
unsafe fn symbol(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |_: ()| STATE.symbol())
}

#[no_mangle]
unsafe fn decimals(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |_: ()| STATE.decimals())
}

#[no_mangle]
unsafe fn total_supply(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |_: ()| STATE.total_supply())
}

#[no_mangle]
unsafe fn account(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |arg| STATE.account(arg))
}

#[no_mangle]
unsafe fn allowance(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |arg| STATE.allowance(arg))
}

#[no_mangle]
unsafe fn transfer(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |arg| STATE.transfer(arg))
}

#[no_mangle]
unsafe fn transfer_from(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |arg| STATE.transfer_from(arg))
}

#[no_mangle]
unsafe fn approve(arg_len: u32) -> u32 {
    rusk_abi::wrap_call(arg_len, |arg| STATE.approve(arg))
}
