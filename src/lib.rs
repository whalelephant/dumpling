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
use staking::{
    ActiveEraInfo, ElectionCompute, EraIndex, Exposure, Nominations, StakingLedger, ValidatorPrefs,
};
use std::collections::HashMap;
use substrate_api_client::Api;

/// Dumpling is a simple wrapper around substrate-api-client
pub struct Dumpling {
    pub api: Api<sr25519::Pair>,
}

impl Dumpling {
    /// Create Dumpling with:
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

    // pulse
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

    // validators
    pub fn waiting_validators(
        &self,
        block_hash: Option<Hash>,
    ) -> HashMap<
        AccountId,
        (
            Option<StakingLedger<AccountId, Balance>>,
            Option<ValidatorPrefs>,
        ),
    > {
        let mut waitlist = HashMap::new();
        let key_prefix = self.api.get_storage_map_key_prefix("Staking", "Validators");
        let keys_str = self.api.get_keys(key_prefix, block_hash).unwrap();

        for key in keys_str {
            let storage_key: StorageKey = StorageKey(Vec::from_hex(&key[2..]).unwrap());
            let account_id: AccountId = sr25519::Public::from_slice(&storage_key.0[40..]).into();
            let ledger: Option<StakingLedger<AccountId, Balance>> = self
                .api
                .get_storage_map::<AccountId, _>("Staking", "Ledger", account_id.clone(), None);
            let validator_prefs: Option<ValidatorPrefs> = self
                .api
                .get_storage_by_key_hash::<ValidatorPrefs>(storage_key, None);

            waitlist.insert(account_id, (ledger, validator_prefs));
        }
        waitlist
    }

    // In the PoA phrase, we do not to display ledgers / exposure
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

    //nominators
    pub fn nominators(
        &self,
        block_hash: Option<Hash>,
    ) -> HashMap<String, Option<Nominations<AccountId>>> {
        let key_prefix = self.api.get_storage_map_key_prefix("Staking", "Nominators");
        let keys_str = self.api.get_keys(key_prefix, block_hash).unwrap();
        let mut nominations = HashMap::new();
        for key in keys_str {
            let storage_key: StorageKey = StorageKey(Vec::from_hex(&key[2..]).unwrap());
            let account_id = sr25519::Public::from_slice(&storage_key.0[40..]).to_ss58check();
            let n: Option<Nominations<AccountId>> =
                self.api.get_storage_by_key_hash(storage_key, block_hash);
            nominations.insert(account_id, n);
        }
        nominations
    }
}

// A copy of the ElectionResults from staking to make fields public
#[derive(Decode)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    pub elected_stashes: Vec<AccountId>,
    pub exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    pub compute: ElectionCompute,
}
