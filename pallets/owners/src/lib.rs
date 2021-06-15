#![cfg_attr(not(feature = "std"), no_std)]



// Imports
// -------------------------------------------------

use sp_std::{
	vec::Vec,
	convert::TryInto
};

use frame_support::{
	debug,
	ensure,
	decl_error, decl_event, decl_module, decl_storage,
	traits::{Currency, ExistenceRequirement, Get},
	weights::Weight
};

use frame_system::{
	ensure_signed, ensure_root,
	offchain::{
		AppCrypto,
		SendSignedTransaction,
		CreateSignedTransaction,
		Signer,
		SignMessage
	}
};

use sp_runtime::{
	ModuleId,
	SaturatedConversion,
	RuntimeAppPublic,
	traits::{
		AccountIdConversion
	},
	RuntimeDebug,
	offchain as rt_offchain,
};

use sp_core::{crypto::KeyTypeId};

use sp_io::hashing::{
	keccak_256
};

use parity_scale_codec::{Encode,Decode};

use core::fmt::Debug ;

use hex ;



// Offchain boilerplate

pub const KEY_TYPE: KeyTypeId = KeyTypeId(*b"ownr");

pub mod crypto {
	use crate::KEY_TYPE;
	use sp_core::sr25519::Signature as Sr25519Signature;
	use sp_runtime::app_crypto::{app_crypto, sr25519};
	use sp_runtime::{traits::Verify, MultiSignature, MultiSigner};

	app_crypto!(sr25519, KEY_TYPE);

	pub struct OwnersAppCrypto;

	impl frame_system::offchain::AppCrypto<MultiSigner, MultiSignature> for OwnersAppCrypto {
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}

	impl frame_system::offchain::AppCrypto<<Sr25519Signature as Verify>::Signer, Sr25519Signature>
		for OwnersAppCrypto
	{
		type RuntimeAppPublic = Public;
		type GenericSignature = sp_core::sr25519::Signature;
		type GenericPublic = sp_core::sr25519::Public;
	}
}



// Trait, types and constants used by this pallet
// -------------------------------------------------

pub trait Config: frame_system::Config + CreateSignedTransaction<Call<Self>> {
	type OwnersAppCrypto: AppCrypto<Self::Public, Self::Signature>;
	type OwnersPublic: RuntimeAppPublic + Debug + AsRef<[u8]> ;
	type Call: From<Call<Self>>;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId> ;
	type MaxLengthURL: Get<u8> ;
	type NumChecksRequired: Get<u8> ;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance ;

type PublicOf<T> = <T as Config>::OwnersPublic ;

const PALLET_ID: ModuleId = ModuleId(*b"AUTHORS!");

const HASH_LENGTH: usize = 32 ;

const INTRO_LENGTH: usize = 128 ;

const MARK_LENGTH: usize = 128 ;

const FETCH_TIMEOUT_PERIOD: u64 = 5000 ;

const MARK_PREFIX: &str  = "wika.network/author/" ;

const REVEAL_QUEUE_PREFIX: &[u8] = b"owner/rq";






// Utility functions to convert from different types
// -------------------------------------------------

fn u128_to_balance<T:Config>(input: u128) -> BalanceOf<T> {
	input.saturated_into()
}

fn u8_to_block<T:Config>(input: u8) -> T::BlockNumber {
	input.saturated_into()
}

fn block_to_u32<T:Config>(input: T::BlockNumber) -> u32 {
	match input.try_into() {
		Ok(num) => num,
		Err(_) => 0
	}
}





// Functions to fetch the data from URL
// -------------------------------------------------

fn fetch_from_url(url: &Vec<u8>) -> Option<Vec<u8>> {
	debug::debug!(target: "AUTHOR", "fetch_from_url url: {:?}", url);

	// Convert bytes to str
	let url_str = sp_std::str::from_utf8(url) ;
	if url_str.is_err() {
		debug::debug!(target: "AUTHOR", "fetch_from_url could not convert url bytes to str");
		return None ;
	}
	let url_str = url_str.unwrap() ;
	debug::debug!(target: "AUTHOR", "fetch_from_url url_str: {:?}", url_str);

	// Initiate an external HTTP GET request.
	let request = rt_offchain::http::Request::get(url_str);

	// Setting the timeout.
	let timeout = sp_io::offchain::timestamp()
		.add(rt_offchain::Duration::from_millis(FETCH_TIMEOUT_PERIOD));

	// Sending the request
	let pending = request
		.deadline(timeout)
		.send() ;
	if pending.is_err() {
		debug::debug!(target: "AUTHOR", "fetch_from_url failed to send the request");
		return None ;
	}
	let pending = pending.unwrap() ;

	// The returning value here is a `Result` of `Result`,
	let response = pending.try_wait(timeout) ;

	// Unwrap twice
	if response.is_err() {
		debug::debug!(target: "AUTHOR", "fetch_from_url failed to wait for the response");
		return None ;
	}
	let response = response.unwrap() ;
	if response.is_err() {
		debug::debug!(target: "AUTHOR", "fetch_from_url failed to fetch the response");
		return None ;
	}
	let response = response.unwrap() ;
	debug::debug!(target: "AUTHOR", "fetch_from_url response code: {:?}", response.code);

	// Make sure we have a 200
	if response.code != 200 {
		debug::debug!(target: "AUTHOR", "fetch_from_url bad response");
		return None ;
	}

	// Next we fully read the response body and convert it to str
	let bytes = response.body().collect::<Vec<u8>>() ;
	Some(bytes)
}







// Persistent data
// -------------------------------------------------

decl_storage! {
	trait Store for Module<T: Config> as Owners {
		// Total number of URLs registered
		UrlCount: u128 = 0 ;

		// Price for one URL check
    	RequestPrice: u128 = 5_000_000_000_000;

    	// Number of blocks after the request
    	// during which commits are allowed
    	NumBlocksToCommit: u8 = 5 ;

    	// Number of blocks after the end of commits
    	// during which reveals are allowed
    	NumBlocksToReveal: u8 = 5 ;

		// Number of blocks after the end of reveals
    	// during which request data is persisted
    	// After this time, requests, commits and reveals
    	// are deleted
    	NumBlocksToDelete: u8 = 5 ;

    	// Registered verifiers
    	// 1. Block at which they were registered
    	// 2. Enabled true/false
    	// 2. Array of stats in following order
    	//stats[0]: number of commits
		//stats[1]: total blocks waited to commit
		//stats[2]: number of reveals
		//stats[3]: total blocks waited to reveal, after commits were closed
		//stats[4]: number of valid reveals
		//stats[5]: number of YES votes
		//stats[6]: number of NO votes
		//stats[7]: number of votes against the majority
    	Verifiers: map hasher(identity) T::AccountId => (T::BlockNumber, bool, [u32; 8]) ;

    	// List of requests received by block
    	History: map hasher(identity) T::BlockNumber => Vec<Vec<u8>> ;

    	// Request data:
    	// - Block number
    	// - Account
    	Requests: map hasher(blake2_128_concat) Vec<u8> => (T::BlockNumber, T::AccountId) ;

    	// Commit data
    	// Should be the keccak_256 of the concatenation
    	// of the params that will be sent to reveal
    	// separated by commas
    	// example: "0,xxx,yyy,zzz"
    	Commits: double_map hasher(blake2_128_concat) Vec<u8>, hasher(identity) T::AccountId => [u8; 32] ;

    	// Reveal data
    	// - Vote Yes or No
    	// - keccak_256 of the first 128 characters of the webpage
    	// - keccak_256 of the 128 characters containing the mark
    	Reveals: double_map hasher(blake2_128_concat) Vec<u8>, hasher(identity) T::AccountId => (bool, [u8; 32], [u8; 32]) ;

    	// Final URL-Account map representing ownership
    	Authors: map hasher(blake2_128_concat) Vec<u8> => T::AccountId ;
	}
}







// Events
// -------------------------------------------------

decl_event!(
	/// Events generated by the module.
	pub enum Event<T>
	where
		AccountId = <T as frame_system::Config>::AccountId,
	{
		VerifierAdded(AccountId),
    	VerifierEnabled(AccountId),
    	VerifierDisabled(AccountId),
        UrlCheckRequested(AccountId, Vec<u8>),
        UrlCheckCommitted(AccountId, Vec<u8>),
        UrlCheckRevealed(AccountId, Vec<u8>),
	}
);




// Errors
// -------------------------------------------------

decl_error! {
	pub enum Error for Module<T: Config> {
        // 0
        UrlTooLong,

        // 1
        NotEnoughBalanceToRequestUrlCheck,

        // 2
        UrlCheckAlreadyInQueue,

        // 3
        UrlCheckNotFound,

        // 4
        OffTimeToCommit,

        // 5
        OffTimeToReveal,

        // 6
        ExpectedHashWith32Bytes,

        // 7
        InvalidProofOfOwnership,

        // 8
        MismatchBetweenCommitAndReveal,

        // 9
        VerifierAlreadyRegistered,

        // 10
        VerifierNotRegistered,

        // 11
        OffchainSignedTxError,

        // 12
        NoLocalAcctForSigning,

        // 13
        InvalidSalt
	}
}






// Implementation
// -------------------------------------------------


impl<T: Config> Module<T> {

	fn pot_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

	pub fn pot() -> BalanceOf<T> {
		T::Currency::free_balance(&Self::pot_id())
	}

	fn is_verifier_registered(who: &T::AccountId) -> bool {
		Verifiers::<T>::contains_key(who)
	}

	fn is_verifier_enabled(who: &T::AccountId) -> bool {
		if Verifiers::<T>::contains_key(who) {
			Verifiers::<T>::get(who).1
		} else {
			false
		}
	}

	fn is_url_being_verified(url: &Vec<u8>) -> bool {
		Requests::<T>::contains_key(url)
	}

	fn create_request(block: &T::BlockNumber, url: &Vec<u8>, sender: &T::AccountId) {
		let mut urls = History::<T>::take(block);
		urls.push(url.clone());
		History::<T>::insert(block, urls);
		Requests::<T>::insert(url, (block,sender)) ;
	}

	fn send_to_pot(sender: &T::AccountId, amount: BalanceOf<T>) {
		debug::debug!(target: "OWNERS", "Sending likes to pot: {:?}", amount);
		T::Currency::transfer(sender,
							  &Self::pot_id(),
							  amount,
							  ExistenceRequirement::KeepAlive).expect("balance was already checked");
	}

	fn validate_votes(block_number: T::BlockNumber) {

	}

	fn process_votes(block_number: T::BlockNumber) {

	}

	fn delete_requests(block_number: T::BlockNumber) {

	}

	fn am_i_verifier() -> Option<PublicOf<T>> {
		let keys: Vec<PublicOf<T>> = PublicOf::<T>::all() ;
		for x in keys {
			let bytes: [u8; 32] = x.as_ref().try_into().expect("cant fail") ;
			let account: T::AccountId = T::AccountId::decode(&mut &bytes[..]).expect("never fails") ;
			debug::debug!(target: "OWNERS", "offchain_worker account: {:?}", &account);
			if Self::is_verifier_enabled(&account) {
				return Some(x) ;
			}
		}
		return None ;
	}

	fn check_url_offchain(url: &Vec<u8>, requester: &T::AccountId, requested_at: T::BlockNumber) {
		debug::debug!(target: "OWNERS", "check_url_offchain: {:?}", url);

		// Fetch data from url
		let bytes = fetch_from_url(url) ;
		if bytes.is_none() {
			debug::debug!(target: "OWNERS", "check_url_offchain could not fetch data from url");
			return ;
		}
		let bytes = bytes.unwrap() ;

		// Convert to str
		let data = sp_std::str::from_utf8(&bytes) ;
		if data.is_err() {
			debug::debug!(target: "OWNERS", "check_url_offchain could not convert bytes to str");
			return ;
		}
		let data = data.unwrap() ;

		// Intro part
		let intro = &data[..INTRO_LENGTH] ;
		debug::debug!(target: "OWNERS", "check_url_offchain intro: {:?}", intro);
		let intro: Vec<u8> = intro.into() ;

		// Mark part
		let mark_idx = data.find(MARK_PREFIX) ;
		if mark_idx.is_none() {
			debug::debug!(target: "OWNERS", "check_url_offchain mark not found, voting NO");
			Self::send_commit_offchain(url, requested_at, false, &intro, None) ;
			return ;
		}
		let mark_idx = mark_idx.unwrap() ;
		let mark_str = &data[mark_idx..mark_idx+MARK_LENGTH] ;
		debug::debug!(target: "OWNERS", "check_url_offchain mark_str: {:?}", mark_str);

		// Check that the mark contains the address
		debug::debug!(target: "OWNERS", "check_url_offchain requester: {:?}", &requester);
		let address: [u8; 32] = requester.encode().try_into().expect("address is always 32") ;
		let mut address_hex: [u8; 64] = [0; 64] ;
		let conversion = hex::encode_to_slice(address, &mut address_hex) ;
		if conversion.is_err() {
			debug::debug!(target: "OWNERS", "check_url_offchain could not convert address to hex");
			return ;
		}
		let address = sp_std::str::from_utf8(&address_hex) ;
		if address.is_err() {
			debug::debug!(target: "OWNERS", "check_url_offchain could not convert address to str");
			return ;
		}
		let address = address.unwrap() ;
		debug::debug!(target: "OWNERS", "check_url_offchain address: {:?}", &address);
		let address_idx = mark_str.find(&address) ;
		if address_idx.is_none() {
			debug::debug!(target: "OWNERS", "check_url_offchain mark address does not match, voting NO");
			Self::send_commit_offchain(url, requested_at, false, &intro, None) ;
			return ;
		}

		// Valid mark found, let's vote YES
		let proof: Vec<u8> = mark_str.into() ;
		debug::debug!(target: "OWNERS", "check_url_offchain voting YES");
		&Self::send_commit_offchain(url, requested_at, true, &intro, Some(&proof)) ;
	}

	fn concat_data1(vote: bool, intro: &Vec<u8>, proof: Option<&Vec<u8>>) -> Vec<u8> {
		let mut ans: Vec<u8> = sp_std::vec![] ;
		if vote {
			ans.push(b'1') ;
		} else {
			ans.push(b'0') ;
		}
		ans.push(b',') ;
		ans.append(&mut intro.clone()) ;
		if proof.is_some() {
			ans.push(b',') ;
			ans.append(&mut proof.unwrap().clone()) ;
		}
		ans
	}

	fn concat_data2(vote: bool, intro: &Vec<u8>, proof: Option<&Vec<u8>>, salt: &Vec<u8>) -> Vec<u8> {
		let mut ans: Vec<u8> = sp_std::vec![] ;
		if vote {
			ans.push(b'1') ;
		} else {
			ans.push(b'0') ;
		}
		ans.push(b',') ;
		ans.append(&mut intro.clone()) ;
		if proof.is_some() {
			ans.push(b',') ;
			ans.append(&mut proof.unwrap().clone()) ;
		}
		ans.push(b',') ;
		ans.append(&mut salt.clone()) ;
		ans
	}

	fn send_commit_offchain(url: &Vec<u8>, requested_at: T::BlockNumber, vote: bool, intro: &Vec<u8>, proof: Option<&Vec<u8>>) {
		// Concatenate the 3 parameters
		let concat1: Vec<u8> = Self::concat_data1(vote, intro, proof) ;

		// Sign this part to get the salt
		let signer = Signer::<T, T::OwnersAppCrypto>::any_account();
		let sign = signer.sign_message(&concat1) ;
		if sign.is_none() {
			debug::debug!(target: "OWNERS", "send_commit_offchain unable to sign the parameters");
			return ;
		}
		let (_account, salt) = sign.unwrap() ;
		let salt: Vec<u8> = salt.encode() ;
		debug::debug!(target: "OWNERS", "send_commit_offchain salt: {:?}", &salt);

		// Concatenate all 4 params now
		let concat2: Vec<u8> = Self::concat_data2(vote, intro, proof, &salt) ;

		// Generate the hash
		let commit_hash: [u8; 32] = keccak_256(&concat2);
		let commit_hash: Vec<u8> = commit_hash.into() ;


		// Check that it's still time to commit
		let current_block = Self::current_block_number() ;
		debug::error!(target: "OWNERS", "send_commit_offchain current_block: {:?}", current_block);
		let param = u8_to_block::<T>(NumBlocksToCommit::get()) ;
		let max_block = requested_at + param ;
		if current_block>=max_block {
			debug::debug!(target: "OWNERS", "send_commit_offchain too late to commit");
			return ;
		}
		let reveal_at = max_block + u8_to_block::<T>(1) ;

		// Submit the commit transaction
		let result = signer.send_signed_transaction(|_acct| Call::commit_verification(url.clone(), commit_hash.clone()));
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!(target: "OWNERS", "send_commit_offchain TRANSACTION FAILED. account id: {:?}", acc.id);
			} else {
				debug::debug!(target: "OWNERS", "send_commit_offchain SUCCESS");
				Self::save_to_reveal_queue(url, reveal_at, vote, intro, proof, &salt) ;
			}
		} else {
			debug::error!(target: "OWNERS", "send_commit_offchain No local account to submit commit transaction");
		}
	}

	fn save_to_reveal_queue(url: &Vec<u8>, reveal_at: T::BlockNumber, vote: bool, intro: &Vec<u8>, proof: Option<&Vec<u8>>, salt: &Vec<u8>) {

	}

	fn send_reveal_offchain(block_number: T::BlockNumber) -> Result<(), Error<T>> {
		// We retrieve a signer and check if it is valid.
		//   Since this pallet only has one key in the keystore. We use `any_account()1 to
		//   retrieve it. If there are multiple keys and we want to pinpoint it, `with_filter()` can be chained,
		//   ref: https://substrate.dev/rustdocs/v3.0.0/frame_system/offchain/struct.Signer.html
		let signer = Signer::<T, T::OwnersAppCrypto>::any_account();
		let can_sign = signer.can_sign() ;
		debug::debug!(target: "OWNERS", "send_reveal_offchain can_sign: {:?}", can_sign);

		// Translating the current block number to number and submit it on-chain
		let number: u64 = block_number.try_into().unwrap_or(0);

		// `result` is in the type of `Option<(Account<T>, Result<(), ()>)>`. It is:
		//   - `None`: no account is available for sending transaction
		//   - `Some((account, Ok(())))`: transaction is successfully sent
		//   - `Some((account, Err(())))`: error occured when sending the transaction
		let result = signer.send_signed_transaction(|_acct| Call::test_tx(number));

		// Display error if the signed tx fails.
		if let Some((acc, res)) = result {
			if res.is_err() {
				debug::error!("failure: offchain_signed_tx: tx sent: {:?}", acc.id);
				return Err(<Error<T>>::OffchainSignedTxError);
			}
			// Transaction is sent successfully
			return Ok(());
		} else {
			// The case result == `None`: no account is available for sending
			debug::error!("No local account available");
			return Err(<Error<T>>::NoLocalAcctForSigning);
		}
	}

	fn current_block_number() -> T::BlockNumber {
		<frame_system::Module<T>>::block_number()
	}

	fn key_reveals_for_block(block_number: T::BlockNumber) -> Vec<u8> {
		block_number.using_encoded(|encoded_bn| {
			REVEAL_QUEUE_PREFIX.clone().into_iter()
				.chain(b"/".into_iter())
				.chain(encoded_bn)
				.copied()
				.collect::<Vec<u8>>()
		})
	}

}


decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		// Process previous requests
		fn on_initialize(current_block: T::BlockNumber) -> Weight {
			debug::debug!(target: "OWNERS", "on_initialize processing previous requests...");
			&Self::validate_votes(current_block) ;
			&Self::process_votes(current_block) ;
			&Self::delete_requests(current_block) ;
			100_000
		}

		// Test Tx
        #[weight = 10_000]
        fn test_tx(origin, number:u64) {
            // Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;
			debug::debug!(target: "OWNERS", "test_tx sender: {:?}", &sender);
			debug::debug!(target: "OWNERS", "test_tx number: {:?}", &number);
        }

		// Add a validator
        #[weight = 10_000]
        fn add_verifier(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is not already registered
			ensure!(!Self::is_verifier_registered(&account), Error::<T>::VerifierAlreadyRegistered) ;

			// Add account as a new verifier
			let current_block = <frame_system::Module<T>>::block_number();
			let stats:[u32;8] = [0;8];
			let verifier = (current_block, true, stats) ;
			Verifiers::<T>::insert(&account, verifier);

            // Emit an event that new validator was added.
            Self::deposit_event(RawEvent::VerifierAdded(account));
        }

        // Disable a verifier
        #[weight = 10_000]
        fn disable_verifier(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is already registered
			ensure!(Self::is_verifier_registered(&account), Error::<T>::VerifierNotRegistered) ;

			// Disable account
			let mut verifier = Verifiers::<T>::take(&account) ;
			verifier.1 = false ;
			Verifiers::<T>::insert(&account, &verifier) ;

            // Emit an event that new validator was added.
            Self::deposit_event(RawEvent::VerifierDisabled(account));
        }

        // Enable a verifier
        #[weight = 10_000]
        fn enable_verifier(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is already in the list
			ensure!(Self::is_verifier_registered(&account), Error::<T>::VerifierNotRegistered) ;

			// Enable account
			let mut verifier = Verifiers::<T>::take(&account) ;
			verifier.1 = true ;
			Verifiers::<T>::insert(&account, &verifier) ;

            // Emit an event that verifier was enabled.
            Self::deposit_event(RawEvent::VerifierEnabled(account));
        }

        // Trigger a new url check
        #[weight = 10_000]
        fn request_url_check(origin, url: Vec<u8>) {
            // Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;

			// Check URL length
			ensure!(url.len()<T::MaxLengthURL::get().into(), Error::<T>::UrlTooLong) ;

			// Check that the signer has enough funds to be sent to the pot
			let price = u128_to_balance::<T>(RequestPrice::get()) ;
			let free = T::Currency::free_balance(&sender) ;
			ensure!(free>price, Error::<T>::NotEnoughBalanceToRequestUrlCheck) ;

			// Check that that this URL is not already in the queue
			ensure!(!Self::is_url_being_verified(&url), Error::<T>::UrlCheckAlreadyInQueue) ;

			// Send check price to pot
			Self::send_to_pot(&sender, price) ;

			// Insert the URL in the check request queue at current block
			let current_block = <frame_system::Module<T>>::block_number();
			Self::create_request(&current_block, &url, &sender) ;
			debug::debug!(target: "OWNERS", "request_url_check inserted at block: {:?}", &current_block);

            // Emit an event that UrlCheckRequest was recorded.
            Self::deposit_event(RawEvent::UrlCheckRequested(sender, url));
        }

        // Receive commits from verfiers
        #[weight = 10_000]
        fn commit_verification(origin, url: Vec<u8>, hash: Vec<u8>) {
        	// Print params for debugginng purposes
        	debug::debug!(target: "OWNERS", "commit_verification url: {:?}", &url);
        	debug::debug!(target: "OWNERS", "commit_verification hash: {:?}", &hash);

            // Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;

			// Check the hash length
			ensure!(hash.len()==HASH_LENGTH, Error::<T>::ExpectedHashWith32Bytes) ;

			// Check that the signer is an active verifier
			ensure!(Self::is_verifier_enabled(&sender), Error::<T>::VerifierNotRegistered) ;

			// Check that the request exists in the queue
			ensure!(Requests::<T>::contains_key(&url), Error::<T>::UrlCheckNotFound) ;

			// Check that it's a good time to receive commits
			let current_block = Self::current_block_number() ;
			let request_block = Requests::<T>::get(&url).0 ;
			let param = u8_to_block::<T>(NumBlocksToCommit::get()) ;
			let max_block = request_block + param ;
			let timing_ok = current_block>request_block && current_block<max_block ;
			debug::debug!(target: "OWNERS", "commit_verification current_block: {:?}", &current_block);
			debug::debug!(target: "OWNERS", "commit_verification request_block: {:?}", &request_block);
			debug::debug!(target: "OWNERS", "commit_verification max_block: {:?}", &max_block);
			ensure!(timing_ok, Error::<T>::OffTimeToCommit) ;

			// Save the commit
			let hash_array: [u8; 32] = hash.try_into().expect("length already checked") ;
			Commits::<T>::insert(&url, &sender, hash_array);
			debug::debug!(target: "OWNERS", "commit_verification commit saved!");

			// Update verifier stats
			let mut stats = Verifiers::<T>::take(&sender);
			stats.2[0] += 1 ;
			let n_blocks:u32 = block_to_u32::<T>(current_block-request_block)  ;
			stats.2[1] += n_blocks ;
			Verifiers::<T>::insert(&sender, &stats);
			debug::debug!(target: "OWNERS", "commit_verification updated stats: {:?}", &stats);

            // Emit an event that the commit was recorded.
            Self::deposit_event(RawEvent::UrlCheckCommitted(sender, url));
        }

		// Receive reveals from verifiers
		#[weight = 10_000]
        fn reveal_verification(origin, url: Vec<u8>,
        					   vote: bool, intro: Vec<u8>, proof: Vec<u8>, salt: Vec<u8>) {
        	// Print params for debugging purposes
			debug::debug!(target: "OWNERS", "reveal_verification url: {:?}", &url);
        	debug::debug!(target: "OWNERS", "reveal_verification vote: {:?}", &vote);

        	// Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;

			// Check that the signer is an enabled verifier
			ensure!(Self::is_verifier_enabled(&sender), Error::<T>::VerifierNotRegistered) ;

			// Check that the request exists in the queue
			ensure!(Requests::<T>::contains_key(&url), Error::<T>::UrlCheckNotFound) ;
			let request = Requests::<T>::get(&url) ;
			let request_block = request.0 ;
			let request_account = request.1 ;

			// If vote is positive, check that the proof contains the account address
			if vote {
				debug::debug!(target: "OWNERS", "reveal_verification request_account: {:?}", &request_account);
				debug::debug!(target: "OWNERS", "reveal_verification proof: {:?}", &proof);
				// TODO
				ensure!(true, Error::<T>::InvalidProofOfOwnership) ;
			}

			// Check that it's a good time to receive reveals
			let current_block = <frame_system::Module<T>>::block_number();
			let param1 = u8_to_block::<T>(NumBlocksToCommit::get()) ;
			let param2 = u8_to_block::<T>(NumBlocksToReveal::get()) ;
			let min_block = request_block + param1 ;
			let max_block = request_block + param1 + param2 ;
			let timing_ok = current_block>min_block && current_block<max_block ;
			debug::debug!(target: "OWNERS", "reveal_verification current_block: {:?}", &current_block);
			debug::debug!(target: "OWNERS", "reveal_verification min_block: {:?}", &min_block);
			debug::debug!(target: "OWNERS", "reveal_verification max_block: {:?}", &max_block);
			ensure!(timing_ok, Error::<T>::OffTimeToReveal) ;

            // Check that the result was previously committed
            ensure!(Commits::<T>::contains_key(&url, &sender), Error::<T>::VerifierNotRegistered) ;

            // Check that the salt is a valid signature
            let account_bytes: [u8; 32] = sender.encode().try_into().expect("account len is 32") ;
            let account_public: sp_core::sr25519::Public = sp_core::sr25519::Public::from_raw(account_bytes) ;
			let signature_bytes: Result<[u8; 64],_> = salt.clone().try_into() ;
			ensure!(signature_bytes.is_ok(), Error::<T>::InvalidSalt) ;

			// Check that the salt is actually the signature for the first 3 params
			let signature_bytes: [u8; 64] = signature_bytes.unwrap() ;
            let signature = sp_core::sr25519::Signature::from_raw(signature_bytes) ;
			let proof_option = match vote {
            	true => Some(&proof),
            	false => None
            } ;
            let concat1 = Self::concat_data1(vote, &intro, proof_option) ;
            debug::debug!(target: "OWNERS", "reveal_verification concat: {:?}", &concat1);
			let valid_salt = sp_io::crypto::sr25519_verify(&signature, &concat1, &account_public) ;
			ensure!(valid_salt, Error::<T>::InvalidSalt) ;

			// Check that the reveal is consistent with commit
			let concat2 = Self::concat_data2(vote, &intro, proof_option, &salt) ;
            let reveal_hash = keccak_256(&concat2);
            debug::debug!(target: "OWNERS", "reveal_verification reveal_hash: {:?}", &reveal_hash);
            let commit = Commits::<T>::get(&url, &sender) ;
            ensure!(reveal_hash==commit, Error::<T>::MismatchBetweenCommitAndReveal) ;

			// Save the reveal
			let intro_hash = keccak_256(&intro);
			let proof_hash = keccak_256(&proof);
			Reveals::<T>::insert(&url, &sender, (vote, intro_hash, proof_hash));

			// Update verifier stats
			let mut stats = Verifiers::<T>::take(&sender);
			stats.2[2] += 1 ;
			let n_blocks:u32 = block_to_u32::<T>(current_block-min_block) ;
			stats.2[3] += n_blocks ;
			Verifiers::<T>::insert(&sender, &stats);

            // Emit an event that the commit was recorded.
            Self::deposit_event(RawEvent::UrlCheckRevealed(sender, url));
		}


        // Offchain Worker:
        // - Process the requests of the block and send commits
        // - Send reveals when it's time
		fn offchain_worker(block_number: T::BlockNumber) {
			debug::debug!(target: "OWNERS", "offchain_worker checking node account");
			let account = Self::am_i_verifier() ;
			if account.is_none() {
				debug::debug!(target: "OWNERS", "offchain_worker is OFF");
				return ;
			}
			debug::debug!(target: "OWNERS", "offchain_worker is ON");
			let requests = History::<T>::get(block_number) ;
			for url in requests.iter() {
				let request = Requests::<T>::get(&url) ;
				let requested_at = request.0 ;
				let requester = request.1 ;
				Self::check_url_offchain(&url, &requester, requested_at) ;
			}
			//let signer = Signer::<T, T::OwnersAppCrypto>::any_account();
			//let number: u64 = 123 ;
			//let result = signer.send_signed_transaction(|_acct| Call::test_tx(number));
		}

	}
}

