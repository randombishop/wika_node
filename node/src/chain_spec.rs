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


// The URL for the telemetry server.
// const STAGING_TELEMETRY_URL: &str = "wss://telemetry.polkadot.io/submit/";

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

pub fn authority_keys_from_address(s: &[u8; 32]) -> (AuraId, GrandpaId) {

	(
		get_from_address::<AuraId>(s),
		get_from_address::<GrandpaId>(s),
	)
}




// Convert list of addresses to AccountID or authorities

fn list_to_accounts(list: &Vec<[u8; 32]>) -> Vec<AccountId> {
	let mut ans: Vec<AccountId> = vec![] ;
	for addr in list {
		ans.push(get_account_id_from_address::<sr25519::Public>(&addr)) ;
	}
	ans
}

fn list_to_authorities(list: &Vec<[u8; 32]>) -> Vec<(AuraId, GrandpaId)> {
	let mut ans: Vec<(AuraId, GrandpaId)> = vec![] ;
	for addr in list {
		ans.push(authority_keys_from_address(&addr)) ;
	}
	ans
}



// TEST and MAIN initial node accounts

const BALANCE_UNIT: u128 = 1000000000000 ;

const INITIAL_AUTHORITIES_BALANCE: u128 = 1000 * BALANCE_UNIT;

fn initial_test_nodes() -> Vec<[u8; 32]> {
	vec![
		hex!("a4f40f452d0de9f15f5402d14cd6379865162ec2263bd630284cf02853d9263c"), //node1
		hex!("a273fb4a9e52dc63c64589bdc9455147581ef0fd1b6420c8bd6f50c22a04a805"), //node2
		hex!("d01e386e8850ac34571f051361c98fb46ead33223bc3d3efa34b453ee987fb38"), //node3
		hex!("3e96986e5a5bc0b64108f9e3a7ea850ff37debebf053ecb9eb58d8e3c14d8307"), //node4
		hex!("52f5dbe7eb64d897f038ef3e9e3f49f944366e054b0e23902ead5d71a7147844"), //node5
		hex!("9ebf4c489c5de1132f90ddd4d1fada40dbea1e7f99b9e30f0cf300728dbd6010")  //sudo
	]
}

fn initial_main_nodes() -> Vec<[u8; 32]> {
	vec![
		hex!("a4f808550b9431df7a27c76e9d736a876f28b029a4b5da497643268f058d0578"), //node1
		hex!("989c9912e45b1c95ddb4145b1fb11285f2679e36a7433d10d03ad629280c5c43"), //node2
		hex!("6ed8182452a29387f0b3a99cf7a5146522f190637400a97429a55158193f603b"), //node3
		hex!("369f8a73a279bd4a840175c31d9c03976389da3d34ac6ab87bdb205c6b156d5f"), //node4
		hex!("0e99f4d8406a1c40a6843e4dd130bb700717e905e88252ef9f2ababf111a6266"), //node5
		hex!("a47f12db28111cfc0052494f434f4bf2cd316dcd13852d6f64e538d861c23534")  //sudo
	]
}






pub fn dev_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;

	Ok(ChainSpec::from_genesis(
		// Name
		"Wika Development Environment",
		// ID
		"dev",
		ChainType::Development,
		move || wika_genesis(
			wasm_binary,
			// Initial PoA authorities
			vec![
				authority_keys_from_seed("Alice"),
				authority_keys_from_seed("Bob")
			],
			// Sudo account
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			// Pre-funded accounts
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
		ChainType::Development,
		move || wika_genesis(
			wasm_binary,
			list_to_authorities(&test_nodes),
			get_account_id_from_address::<sr25519::Public>(&hex!("9ebf4c489c5de1132f90ddd4d1fada40dbea1e7f99b9e30f0cf300728dbd6010")),
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
		// Name
		"Wika MainNet",
		// ID
		"main",
		ChainType::Local,
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
