use execution_core::signatures::bls::{PublicKey, SecretKey};
use execution_core::{ContractId, StandardBufSerializer};
use rusk_abi::{CallReceipt, ContractData, PiecrustError, Session};

use bytecheck::CheckBytes;
use rkyv::validation::validators::DefaultValidator;
use rkyv::{Archive, Deserialize, Infallible, Serialize};

use rand::rngs::StdRng;
use rand::SeedableRng;

use ttoken_types::*;

const BYTECODE: &[u8] = include_bytes!("../../build/ttoken_contract.wasm");
const OWNER: [u8; 64] = [0u8; 64];
const INITIAL_BALANCE: u64 = 1000;

type Result<T, Error = PiecrustError> = std::result::Result<T, Error>;

struct ContractSession {
    deploy_pk: PublicKey,
    deploy_sk: SecretKey,
    contract: ContractId,
    session: Session,
}

impl ContractSession {
    fn new() -> Self {
        let vm = rusk_abi::new_ephemeral_vm().expect("Creating VM should succeed");
        let mut session = rusk_abi::new_genesis_session(&vm);

        let mut rng = StdRng::seed_from_u64(0xF0CACC1A);
        let deploy_sk = SecretKey::random(&mut rng);
        let deploy_pk = PublicKey::from(&deploy_sk);

        let contract = session
            .deploy(
                BYTECODE,
                ContractData::builder()
                    .owner(OWNER)
                    .constructor_arg(&(deploy_pk, INITIAL_BALANCE)),
                u64::MAX,
            )
            .expect("Deploying the contract should succeed");

        Self {
            deploy_sk,
            deploy_pk,
            contract,
            session,
        }
    }

    fn deploy_pk(&self) -> PublicKey {
        self.deploy_pk
    }

    fn call<A, R>(&mut self, fn_name: &str, fn_arg: &A) -> Result<CallReceipt<R>>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible> + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        self.session.call(self.contract, fn_name, fn_arg, u64::MAX)
    }

    fn account(&mut self, pk: &PublicKey) -> Account {
        self.call("account", pk)
            .expect("Querying an account should succeed")
            .data
    }
}

#[test]
fn deploy() {
    ContractSession::new();
}

#[test]
fn empty_account() {
    let mut session = ContractSession::new();

    let mut rng = StdRng::seed_from_u64(0xBEEF);
    let sk = SecretKey::random(&mut rng);
    let pk = PublicKey::from(&sk);

    let account = session.account(&pk);
    assert_eq!(
        account,
        Account::EMPTY,
        "An account never transferred to should be empty"
    );
}

#[test]
fn transfer() {
    const TRANSFERRED_AMOUNT: u64 = INITIAL_BALANCE / 2;

    let mut session = ContractSession::new();

    let mut rng = StdRng::seed_from_u64(0xBEEF);
    let sk = SecretKey::random(&mut rng);
    let pk = PublicKey::from(&sk);

    assert_eq!(
        session.account(&session.deploy_pk()).balance,
        INITIAL_BALANCE,
        "The deployed account should have the initial balance"
    );
    assert_eq!(
        session.account(&pk).balance,
        0,
        "The account to transfer to should have no balance"
    );

    let transfer = Transfer::new(&session.deploy_sk, pk, 500, 1);
    session
        .call::<_, ()>("transfer", &transfer)
        .expect("Transferring should succeed");

    assert_eq!(
        session.account(&session.deploy_pk()).balance,
        INITIAL_BALANCE - TRANSFERRED_AMOUNT,
        "The deployed account should"
    );
    assert_eq!(
        session.account(&pk).balance,
        TRANSFERRED_AMOUNT,
        "The account transferred to should have the transferred amount"
    );
}

fn main() {
    unreachable!("`main` should never run for this crate");
}
