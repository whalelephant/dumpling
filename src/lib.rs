use codec::{Decode, HasCompact};
use hex::FromHex;
use polkadot_primitives::{Balance, BlockNumber};
pub use sp_core::{
    crypto::{Public, Ss58AddressFormat, Ss58Codec},
    sr25519,
    storage::StorageKey,
    H256 as Hash,
};
pub use sp_runtime::{generic::Header, traits::BlakeTwo256, AccountId32 as AccountId};
use sp_staking::SessionIndex;
use staking::{
    ActiveEraInfo, ElectionCompute, EraIndex, Exposure, Nominations, StakingLedger, ValidatorPrefs,
};
use std::collections::HashMap;
use substrate_api_client::Api;

// pulse
pub fn finalized_head() -> (Option<Hash>, Option<Header<BlockNumber, BlakeTwo256>>) {
    let api = client();
    let hash = api.get_finalized_head();
    let header = api.get_header(hash);
    (hash, header)
}

pub fn active_era(block_hash: Option<Hash>) -> Option<ActiveEraInfo> {
    let api = client();
    api.get_storage_value::<ActiveEraInfo>("Staking", "ActiveEra", block_hash)
}

pub fn planned_era(block_hash: Option<Hash>) -> Option<EraIndex> {
    let api = client();
    api.get_storage_value::<EraIndex>("Staking", "CurrentEra", block_hash)
}

pub fn session_index(block_hash: Option<Hash>) -> Option<SessionIndex> {
    let api = client();
    api.get_storage_value::<SessionIndex>("Session", "CurrentIndex", block_hash)
}

// validators
pub fn waiting_validators(
    block_hash: Option<Hash>,
) -> HashMap<
    AccountId,
    (
        Option<StakingLedger<AccountId, Balance>>,
        Option<ValidatorPrefs>,
    ),
> {
    let mut waitlist = HashMap::new();
    let api = client();
    let key_prefix = api.get_storage_map_key_prefix("Staking", "Validators");
    let keys_str = api.get_keys(key_prefix, block_hash).unwrap();

    for key in keys_str {
        let storage_key: StorageKey = StorageKey(Vec::from_hex(&key[2..]).unwrap());
        let account_id: AccountId = sr25519::Public::from_slice(&storage_key.0[40..]).into();
        let ledger: Option<StakingLedger<AccountId, Balance>> =
            api.get_storage_map::<AccountId, _>("Staking", "Ledger", account_id.clone(), None);
        let validator_prefs: Option<ValidatorPrefs> =
            api.get_storage_by_key_hash::<ValidatorPrefs>(storage_key, None);

        waitlist.insert(account_id, (ledger, validator_prefs));
    }
    waitlist
}

// In the PoA phrase, we do not to display ledgers / exposure
pub fn session_validators(block_hash: Option<Hash>) -> Option<Vec<AccountId>> {
    let api = client();
    api.get_storage_value::<Vec<AccountId>>("Session", "Validators", block_hash)
}

// A copy of the ElectionResults from staking to make fields public
#[derive(Decode)]
pub struct ElectionResult<AccountId, Balance: HasCompact> {
    pub elected_stashes: Vec<AccountId>,
    pub exposures: Vec<(AccountId, Exposure<AccountId, Balance>)>,
    pub compute: ElectionCompute,
}

pub fn queued_validators(block_hash: Option<Hash>) -> Option<ElectionResult<AccountId, Balance>> {
    let api = client();
    api.get_storage_value::<ElectionResult<AccountId, Balance>>(
        "Staking",
        "QueuedElected",
        block_hash,
    )
}

//nominators
pub fn nominators(block_hash: Option<Hash>) -> HashMap<String, Option<Nominations<AccountId>>> {
    let api = client();
    let key_prefix = api.get_storage_map_key_prefix("Staking", "Nominators");
    let keys_str = api.get_keys(key_prefix, block_hash).unwrap();
    let mut nominations = HashMap::new();
    for key in keys_str {
        let storage_key: StorageKey = StorageKey(Vec::from_hex(&key[2..]).unwrap());
        let account_id = sr25519::Public::from_slice(&storage_key.0[40..]).to_ss58check();
        let n: Option<Nominations<AccountId>> =
            api.get_storage_by_key_hash(storage_key, block_hash);
        nominations.insert(account_id, n);
    }
    nominations
}

// util function
fn client() -> Api<sr25519::Pair> {
    let node_ip = "127.0.0.1";
    let node_port = "9944";
    let url = format!("{}:{}", node_ip, node_port);
    let ss58_version = Ss58AddressFormat::PolkadotAccount;
    sp_core::crypto::set_default_ss58_version(ss58_version);

    Api::<sr25519::Pair>::new(format!("ws://{}", url))
}
