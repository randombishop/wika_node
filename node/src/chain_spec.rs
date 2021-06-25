use sp_core::{Pair, Public, sr25519};
use wika_runtime::{
	AccountId, BalancesConfig, GenesisConfig,
	SudoConfig, SystemConfig, AuraConfig, GrandpaConfig, AuthoritiesConfig, WASM_BINARY, Signature
};
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



// Get Public and AccountID from addresses.

pub fn get_from_address<TPublic: Public>(address: &[u8; 32]) -> <TPublic::Pair as Pair>::Public {
	<TPublic::Pair as Pair>::Public::from_slice(address)
}

pub fn get_account_id_from_address<TPublic: Public>(address: &[u8; 32]) -> AccountId where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>
{
	AccountPublic::from(get_from_address::<TPublic>(address)).into_account()
}






// Convert list of addresses to AccountID or authorities

fn list_to_accounts(list: &Vec<([u8; 32],[u8; 32])>) -> Vec<AccountId> {
	let mut ans: Vec<AccountId> = vec![] ;
	for (addr, _) in list {
		ans.push(get_account_id_from_address::<sr25519::Public>(&addr)) ;
	}
	ans
}





// TEST and MAIN initial node accounts

const BALANCE_UNIT: u128 = 1000000000000 ;

const INITIAL_AUTHORITIES_BALANCE: u128 = 1000 * BALANCE_UNIT;


fn initial_nodes_dev() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// Alice
		(
			hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
			hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee")
		)
	]
}

fn initial_nodes_test() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// testnode1
		(
			hex!("3277d22306435a4800ebc611216d301cb79d2166b7519a6bdf58b0b6fb523267"),
			hex!("2bdb6dab53650339faa80a06fde1b27ea46cee14ee0523bfc43d6c37d64f933d")
		) ,

		// testnode2
		(
			hex!("d021327f24e3ee188449b9264cb401f94caea467656a505fae1fdb0f6eda523b"),
			hex!("ced4ef1df5f05aabc0ae1f7d5ee499edf765291aa4f5efc0f0b074e059e0ef52")
		)
	]
}

fn initial_nodes_main() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// mainnode1
		(
			hex!("3277d22306435a4800ebc611216d301cb79d2166b7519a6bdf58b0b6fb523267"),
			hex!("2bdb6dab53650339faa80a06fde1b27ea46cee14ee0523bfc43d6c37d64f933d")
		) ,

		// mainnode2
		(
			hex!("d021327f24e3ee188449b9264cb401f94caea467656a505fae1fdb0f6eda523b"),
			hex!("ced4ef1df5f05aabc0ae1f7d5ee499edf765291aa4f5efc0f0b074e059e0ef52")
		)
	]
}






pub fn dev_config() -> Result<ChainSpec, String> {
	let wasm_binary = WASM_BINARY.ok_or_else(|| "Development wasm not available".to_string())?;
	let nodes = initial_nodes_dev() ;

	Ok(ChainSpec::from_genesis(
		"Wika Development Environment",
		"dev",
		ChainType::Development,
		move || wika_genesis(
			wasm_binary,
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			&nodes,
			list_to_accounts(&nodes),
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
	let nodes = initial_nodes_test() ;

	Ok(ChainSpec::from_genesis(
		// Name
		"Wika TestNet",
		// ID
		"test",
		ChainType::Live,
		move || wika_genesis(
			wasm_binary,
			get_account_id_from_address::<sr25519::Public>(&hex!("56442d2ddbace4927f284c799c0e706ce7e0df06f395e2c1bbfb967ece5cf053")),
			&nodes,
			list_to_accounts(&nodes),
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
	let nodes = initial_nodes_main() ;

	Ok(ChainSpec::from_genesis(
		"Wika MainNet",
		"main",
		ChainType::Live,
		move || wika_genesis(
			wasm_binary,
			get_account_id_from_address::<sr25519::Public>(&hex!("a47f12db28111cfc0052494f434f4bf2cd316dcd13852d6f64e538d861c23534")),
			&nodes,
			list_to_accounts(&nodes),
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
	root_key: AccountId,
	initial_authorities: &Vec<([u8; 32],[u8; 32])>,
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
			authorities: vec![],
		}),
		pallet_grandpa: Some(GrandpaConfig {
			authorities: vec![],
		}),
		pallet_sudo: Some(SudoConfig {
			key: root_key,
		}),
		pallet_authorities: Some(AuthoritiesConfig {
			keys: initial_authorities.clone(),
		}),
	}
}
