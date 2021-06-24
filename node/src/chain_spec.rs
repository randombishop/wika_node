use sp_core::{Pair, Public, sr25519};
use wika_runtime::{
	AccountId, AuraConfig, BalancesConfig, GenesisConfig, GrandpaConfig,
	SudoConfig, SystemConfig, AuthoritiesConfig, WASM_BINARY, Signature
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


fn initial_nodes_dev() -> Vec<([u8; 32],[u8; 32])> {
	vec![
		// Alice
		(
			hex!("d43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d"),
			hex!("88dc3417d5058ec4b4503e0c12ea1a0a89be200fe98922423d4334014fa6b0ee")
		),

		// Bob
		(
			hex!("8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48"),
			hex!("d17c2d7823ebf260fd138f2d7e27d114c0145d968b5ff5006125f2414fadae69")
		),

		// Charlie
		(
			hex!("90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22"),
			hex!("439660b36c6c03afafca027b910b4fecf99801834c62a5e6006f27d978de234f")
		),

		// Dave
		(
			hex!("306721211d5404bd9da88e0204360a1a9ab8b87c66c1bc2fcdd37f3c2222cc20"),
			hex!("5e639b43e0052c47447dac87d6fd2b6ec50bdd4d0f614e4299c665249bbd09d9")
		),
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
	let test_nodes = initial_nodes_test() ;

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
	let main_nodes = initial_nodes_main() ;

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
		pallet_authorities: Some(AuthoritiesConfig {
			keys: initial_nodes_dev(),
		}),
	}
}
