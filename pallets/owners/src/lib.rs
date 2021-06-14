#![cfg_attr(not(feature = "std"), no_std)]



// Imports
// -------------------------------------------------

use sp_std::vec::Vec;

use frame_support::{
	debug,
	decl_error, decl_event, decl_module, decl_storage,
};

use frame_system::offchain::{
	AppCrypto,
	SendSignedTransaction,
	CreateSignedTransaction,
	Signer
};

use sp_core::{crypto::KeyTypeId};





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
	type Call: From<Call<Self>>;
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
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



decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		// Test Tx
        #[weight = 10_000]
        fn test_tx(origin, number:u64) {
			debug::debug!(target: "OWNERS", "test_tx");
        }


        // Offchain Worker:
        // - Process the requests of the block and send commits
        // - Send reveals when it's time
		fn offchain_worker(block_number: T::BlockNumber) {
			debug::debug!(target: "OWNERS", "offchain_worker checking node account");

			let signer = Signer::<T, T::OwnersAppCrypto>::any_account();
			let number: u64 = 123 ;
			let result = signer.send_signed_transaction(|_acct| Call::test_tx(number));

		}

	}
}

