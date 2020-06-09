use balances::BalanceLock;
use codec::{Decode, HasCompact};
use hex::FromHex;
use polkadot_primitives::{Balance, BlockNumber};
pub use sp_core::{
    crypto::{Pair, Public, Ss58AddressFormat, Ss58Codec},
    sr25519,
    storage::StorageKey,
    H256 as Hash,
};
pub use sp_runtime::{
    generic::Header, traits::BlakeTwo256, AccountId32 as AccountId, MultiSignature,
};
use sp_staking::SessionIndex;
use staking::{ElectionCompute, EraIndex, Exposure, Nominations, StakingLedger, ValidatorPrefs};
use std::collections::HashMap;
use substrate_api_client::Api;

/// ApiFilling is a simple wrapper around substrate-api-client
///
/// It has methods to easily fetch derived data from key prefix or combined rpc calls
pub struct ApiFilling {
    pub api: Api<sr25519::Pair>,
}

impl ApiFilling {
    /// Create ApiFilling with:
    ///
    /// url string - node_ip:node_port
    /// chain - kusama, polkadot, westend
    pub fn new(url: &str, format: &str) -> Self {
        let ss58_version = match format {
            "polkadot" => Ss58AddressFormat::PolkadotAccount,
            "kusama" => Ss58AddressFormat::KusamaAccount,
            "westend" => Ss58AddressFormat::SubstrateAccount,
            _ => panic!("Format not supported"),
        };
        sp_core::crypto::set_default_ss58_version(ss58_version);

        Self {
            api: Api::<sr25519::Pair>::new(format!("ws://{}", url)),
        }
    }

    pub fn finalized_head(&self) -> (Option<Hash>, Option<Header<BlockNumber, BlakeTwo256>>) {
        let hash = self.api.get_finalized_head();
        let header = self.api.get_header(hash);
        (hash, header)
    }

    pub fn active_era(&self, block_hash: Option<Hash>) -> Option<ActiveEraInfo> {
        self.api
            .get_storage_value::<ActiveEraInfo>("Staking", "ActiveEra", block_hash)
    }

    pub fn planned_era(&self, block_hash: Option<Hash>) -> Option<EraIndex> {
        self.api
            .get_storage_value::<EraIndex>("Staking", "CurrentEra", block_hash)
    }

    pub fn session_index(&self, block_hash: Option<Hash>) -> Option<SessionIndex> {
        self.api
            .get_storage_value::<SessionIndex>("Session", "CurrentIndex", block_hash)
    }

    pub fn waiting_validators(
        &self,
        block_hash: Option<Hash>,
    ) -> HashMap<String, WaitingValidator> {
        let mut waitlist = HashMap::new();
        let key_prefix = self.api.get_storage_map_key_prefix("Staking", "Validators");
        let keys_str = self.api.get_keys(key_prefix, block_hash).unwrap();
        let v_to_n = Self::validators_to_nominators(self, block_hash);

        for key in keys_str {
            let storage_key = Self::string_to_key(&key);
            let account_id = Self::key_to_account(&storage_key);

            let opt_b: Option<Vec<BalanceLock<Balance>>> =
                self.api.get_storage_map::<AccountId, _>(
                    "Balances",
                    "Locks",
                    account_id.clone(),
                    block_hash,
                );
            let mut staked: Balance = 0;
            for i in opt_b.unwrap() {
                if i.id == *b"staking " {
                    staked = i.amount;
                }
            }
            let prefs = self.api.get_storage_by_key_hash(storage_key, None).unwrap();

            let nominators = match v_to_n.get(&account_id) {
                Some(n) => n.clone(),
                None => vec![String::from("None")]
            };

            let ledger = self.api.get_storage_map::<AccountId, _>(
                "Staking",
                "Ledger",
                account_id.clone(),
                None,
            );

            waitlist.insert(
                account_id.to_ss58check(),
                WaitingValidator {
                    staked: staked,
                    prefs: prefs,
                    nominators: nominators,
                    ledger: ledger,
                },
            );
        }
        waitlist
    }

    pub fn validators_to_nominators(
        &self,
        block_hash: Option<Hash>,
    ) -> HashMap<AccountId, Vec<String>> {
        let mut v: HashMap<AccountId, Vec<String>> = HashMap::new();
        let nom_list = Self::get_nominators(self, block_hash);
        for n in nom_list {
            let nom_id = n.1.to_ss58check();
            let nominations = n.2;
            if let Some(n) = nominations {
                for t in n.targets {
                    if let Some(nom_vec) = v.get_mut(&t) {
                        nom_vec.push(nom_id.clone());
                    } else {
                        v.insert(t, vec![nom_id.clone()]);
                    }
                }
            }
        }
        v
    }

    pub fn session_validators(&self, block_hash: Option<Hash>) -> Option<Vec<AccountId>> {
        self.api
            .get_storage_value::<Vec<AccountId>>("Session", "Validators", block_hash)
    }

    pub fn queued_validators(
        &self,
        block_hash: Option<Hash>,
    ) -> Option<ElectionResult<AccountId, Balance>> {
        self.api
            .get_storage_value::<ElectionResult<AccountId, Balance>>(
                "Staking",
                "QueuedElected",
                block_hash,
            )
    }

    pub fn nominators(&self, block_hash: Option<Hash>) -> HashMap<String, Option<Nominator>> {
        let nom_list = Self::get_nominators(self, block_hash);
        let mut nominations = HashMap::new();
        for n in nom_list {
            let account_id = n.1;
            let nom = n.2;
            if let Some(n) = nom {
                let opt_b: Option<Vec<BalanceLock<Balance>>> =
                    self.api.get_storage_map::<AccountId, _>(
                        "Balances",
                        "Locks",
                        account_id.clone(),
                        block_hash,
                    );
                let mut b: Balance = 0;
                for i in opt_b.unwrap() {
                    if i.id == *b"staking " {
                        b = i.amount;
                    }
                }
                nominations.insert(
                    account_id.to_ss58check(),
                    Some(Nominator {
                        nominations: n,
                        staked: b,
                    }),
                );
            } else {
                nominations.insert(account_id.to_ss58check(), None);
            };
        }
        nominations
    }

    fn get_nominators(
        &self,
        block_hash: Option<Hash>,
    ) -> Vec<(StorageKey, AccountId, Option<Nominations<AccountId>>)> {
        let key_prefix = self.api.get_storage_map_key_prefix("Staking", "Nominators");
        let keys_str = self.api.get_keys(key_prefix, block_hash).unwrap();
        let mut nom_list = Vec::new();
        for key in keys_str {
            let storage_key = Self::string_to_key(&key);
            let account_id = Self::key_to_account(&storage_key);
            let nom = self
                .api
                .get_storage_by_key_hash(storage_key.clone(), block_hash);
            nom_list.push((storage_key, account_id, nom));
        }
        nom_list
    }

    fn string_to_key(key: &str) -> StorageKey {
        StorageKey(Vec::from_hex(&key[2..]).unwrap())
    }

    fn key_to_account(s: &StorageKey) -> AccountId {
        sr25519::Public::from_slice(&s.0[40..]).into()
    }
}

pub struct Nominator {
    pub nominations: Nominations<AccountId>,
    pub staked: Balance,
}

pub struct WaitingValidator {
    pub staked: Balance,
    pub prefs: ValidatorPrefs,
    pub nominators: Vec<String>,
    pub ledger: Option<StakingLedger<AccountId, Balance>>,
}

// A copy of the ElectionResults from staking to make fields public
#[derive(Decode)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    pub elected_stashes: Vec<AccountId>,
    pub exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    pub compute: ElectionCompute,
}

// A copy of the ActiveEraInfo from staking to make fields public
#[derive(Decode)]
pub struct ActiveEraInfo {
    /// Index of era.
    pub index: EraIndex,
    /// Moment of start expressed as millisecond from `$UNIX_EPOCH`.
    ///
    /// Start can be none if start hasn't been set for the era yet,
    /// Start is set on the first on_finalize of the era to guarantee usage of `Time`.
    pub start: Option<u64>,
}
