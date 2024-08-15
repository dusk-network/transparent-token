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

    fn call<A, R>(&mut self, fn_name: &str, fn_arg: &A) -> Result<CallReceipt<R>, PiecrustError>
    where
        A: for<'b> Serialize<StandardBufSerializer<'b>>,
        A::Archived: for<'b> CheckBytes<DefaultValidator<'b>>,
        R: Archive,
        R::Archived: Deserialize<R, Infallible> + for<'b> CheckBytes<DefaultValidator<'b>>,
    {
        self.session.call(self.contract, fn_name, fn_arg, u64::MAX)
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

    let account: Account = session
        .call("account", &pk)
        .expect("Querying an account should succeed")
        .data;

    assert_eq!(account, Account::EMPTY);
}

fn main() {
    unreachable!("`main` should never run for this crate");
}
