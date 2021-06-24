use sp_core::{Pair, Public, sr25519};
use wika_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, WASM_BINARY, Signature
};
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{Verify, IdentifyAccount};
use sc_service::ChainType;
use hex_literal::hex;



/// Specialized `ChainSpec`. This is a specialization of the general Substrate ChainSpec type.
pub type ChainSpec = sc_service::GenericChainSpec<GenesisConfig>;


type AccountPublic = <Signature as Verify>::Signer;


// Get Public and AccountID from seed.

pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn authority_keys_from_seed(s: &str) -> (AuraId, GrandpaId) {
	(
		get_from_seed::<AuraId>(s),
		get_from_seed::<GrandpaId>(s),
	)
}


// Get Public and AccountID from addresses.

pub fn get_from_address<TPublic: Public>(address: &[u8; 32]) -> <TPublic::Pair as Pair>::Public {
	<TPublic::Pair as Pair>::Public::from_slice(address)
}

pub fn get_account_id_from_address<TPublic: Public>(address: &[u8; 32]) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_address::<TPublic>(address)).into_account()
}

pub fn authority_keys_from_address(addr_sr25519: &[u8; 32], addr_ed25519: &[u8; 32]) -> (AuraId, GrandpaId) {

	(
		get_from_address::<AuraId>(addr_sr25519),
		get_from_address::<GrandpaId>(addr_ed25519),
	)
}




// Convert list of addresses to AccountID or authorities

fn list_to_accounts(list: &Vec<([u8; 32],[u8; 32])>) -> Vec<AccountId> {
	let mut ans: Vec<AccountId> = vec![] ;
	for (addr, _) in list {
		ans.push(get_account_id_from_address::<sr25519::Public>(&addr)) ;
	}
	ans
}

fn list_to_authorities(list: &Vec<([u8; 32],[u8; 32])>) -> Vec<(AuraId, GrandpaId)> {
	let mut ans: Vec<(AuraId, GrandpaId)> = vec![] ;
	for (addr_sr25519,addr_ed25519) in list {
		ans.push(authority_keys_from_address(&addr_sr25519, &addr_ed25519)) ;
	}
	ans
}



// TEST and MAIN initial node accounts

const BALANCE_UNIT: u128 = 1000000000000 ;

const INITIAL_AUTHORITIES_BALANCE: u128 = 1000 * BALANCE_UNIT;

fn initial_test_nodes() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// node1
		(
			hex!("3277d22306435a4800ebc611216d301cb79d2166b7519a6bdf58b0b6fb523267"),
			hex!("2bdb6dab53650339faa80a06fde1b27ea46cee14ee0523bfc43d6c37d64f933d")
		) ,

		// node2
		(
			hex!("d021327f24e3ee188449b9264cb401f94caea467656a505fae1fdb0f6eda523b"),
			hex!("ced4ef1df5f05aabc0ae1f7d5ee499edf765291aa4f5efc0f0b074e059e0ef52")
		)
	]
}

fn initial_main_nodes() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// node1
		(
			hex!("3277d22306435a4800ebc611216d301cb79d2166b7519a6bdf58b0b6fb523267"),
			hex!("2bdb6dab53650339faa80a06fde1b27ea46cee14ee0523bfc43d6c37d64f933d")
		) ,

		// node2
		(
			hex!("d021327f24e3ee188449b9264cb401f94caea467656a505fae1fdb0f6eda523b"),
			hex!("ced4ef1df5f05aabc0ae1f7d5ee499edf765291aa4f5efc0f0b074e059e0ef52")
		)
	]
}






pub fn dev_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		"Wika Development Environment",
		"dev",
		ChainType::Development,
		move || wika_genesis(
			wasm_binary,
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob")
			],
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			vec![
				get_account_id_from_seed::<sr25519::Public>("Alice"),
				get_account_id_from_seed::<sr25519::Public>("Bob"),
				get_account_id_from_seed::<sr25519::Public>("Charlie"),
				get_account_id_from_seed::<sr25519::Public>("Dave")
			],
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn test_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let test_nodes = initial_test_nodes() ;

	Ok(ChainSpec::from_genesis(
		// Name
		"Wika TestNet",
		// ID
		"test",
		ChainType::Live,
		move || wika_genesis(
			wasm_binary,
			list_to_authorities(&test_nodes),
			get_account_id_from_address::<sr25519::Public>(&hex!("56442d2ddbace4927f284c799c0e706ce7e0df06f395e2c1bbfb967ece5cf053")),
			list_to_accounts(&test_nodes),
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

pub fn main_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let main_nodes = initial_main_nodes() ;

	Ok(ChainSpec::from_genesis(
		"Wika MainNet",
		"main",
		ChainType::Live,
		move || wika_genesis(
			wasm_binary,
			list_to_authorities(&main_nodes),
			get_account_id_from_address::<sr25519::Public>(&hex!("a47f12db28111cfc0052494f434f4bf2cd316dcd13852d6f64e538d861c23534")),
			// Pre-funded accounts
			list_to_accounts(&main_nodes),
			true,
		),
		// Bootnodes
		vec![],
		// Telemetry
		None,
		// Protocol ID
		None,
		// Properties
		None,
		// Extensions
		None,
	))
}

/// Configure initial storage state for FRAME modules.
fn wika_genesis(
	wasm_binary: &[u8],
	initial_authorities: Vec<(AuraId, GrandpaId)>,
	root_key: AccountId,
	endowed_accounts: Vec<AccountId>,
	_enable_println: bool,
) -> GenesisConfig {
	GenesisConfig {
		frame_system: Some(SystemConfig {
			// Add Wasm runtime to storage.
			code: wasm_binary.to_vec(),
			changes_trie_config: Default::default(),
		}),
		pallet_balances: Some(BalancesConfig {
			balances: endowed_accounts.iter().cloned().map(|k|(k, INITIAL_AUTHORITIES_BALANCE)).collect(),
		}),
		pallet_aura: Some(AuraConfig {
			authorities: initial_authorities.iter().map(|x| (x.0.clone())).collect(),
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: initial_authorities.iter().map(|x| (x.1.clone(), 1)).collect(),
		}),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
	}
}
