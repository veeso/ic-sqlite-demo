mod env;

use std::io::Read as _;
use std::path::PathBuf;

use candid::{CandidType, Decode, Principal};
use pocket_ic::nonblocking::PocketIc;
use serde::de::DeserializeOwned;

use crate::TestEnv;
use crate::wasm::Canister;

const DEFAULT_CYCLES: u128 = 2_000_000_000_000_000;

/// Test environment
pub struct PocketIcTestEnv {
    pub pic: PocketIc,
    pub sqlite_db: Principal,
}

impl TestEnv for PocketIcTestEnv {
    fn sqlite_db(&self) -> Principal {
        self.sqlite_db
    }

    async fn query<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let reply = match self.pic.query_call(canister, caller, method, payload).await {
            Ok(result) => result,
            Err(e) => anyhow::bail!("Error calling {}: {:?}", method, e),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }

    async fn update<R>(
        &self,
        canister: Principal,
        caller: Principal,
        method: &str,
        payload: Vec<u8>,
    ) -> anyhow::Result<R>
    where
        R: DeserializeOwned + CandidType,
    {
        let reply = if self.is_live() {
            let id = self
                .pic
                .submit_call(canister, caller, method, payload)
                .await
                .map_err(|e| anyhow::anyhow!("Error submitting call {}: {:?}", method, e))?;
            self.pic.await_call_no_ticks(id).await
        } else {
            self.pic
                .update_call(canister, caller, method, payload)
                .await
        };

        let reply = match reply {
            Ok(r) => r,
            Err(r) => anyhow::bail!("{} was rejected: {:?}", method, r),
        };
        let ret_type = Decode!(&reply, R)?;

        Ok(ret_type)
    }
}

impl PocketIcTestEnv {
    /// Install the canisters needed for the tests
    pub async fn init() -> Self {
        let pic = env::init_pocket_ic()
            .await
            .with_nns_subnet()
            .with_ii_subnet() // To have ECDSA keys
            .with_application_subnet()
            .with_max_request_time_ms(Some(30_000))
            .build_async()
            .await;

        // create canisters
        let sqlite_db = pic.create_canister().await;
        println!("Sqlite DB: {sqlite_db}",);

        Self::install_sqlite_db(&pic, sqlite_db).await;

        Self { sqlite_db, pic }
    }

    pub async fn stop(self) {
        self.pic.drop().await;
    }

    fn is_live(&self) -> bool {
        self.pic.url().is_some()
    }

    /// Install [`Canister::HelloWorld`] canister
    async fn install_sqlite_db(pic: &PocketIc, canister_id: Principal) {
        pic.add_cycles(canister_id, DEFAULT_CYCLES).await;

        let wasm_bytes = Self::load_wasm(Canister::SqliteDb);

        //let init_arg = todo!();
        let init_arg = vec![]; // Encode!(&init_arg).unwrap();

        pic.install_canister(canister_id, wasm_bytes, init_arg, None)
            .await;
    }

    fn load_wasm(canister: Canister) -> Vec<u8> {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push(canister.as_path());

        let mut file = std::fs::File::open(path).unwrap();
        let mut wasm_bytes = Vec::new();
        file.read_to_end(&mut wasm_bytes).unwrap();

        wasm_bytes
    }

    pub async fn live(&mut self, live: bool) {
        if live {
            self.pic.make_live(None).await;
        } else {
            self.pic.stop_live().await;
        }
    }
}
