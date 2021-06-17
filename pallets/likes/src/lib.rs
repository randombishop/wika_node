#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::* ;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error,
	ensure, StorageMap, debug,
	traits::{Currency, ExistenceRequirement, Get}
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use sp_runtime::{
	SaturatedConversion,ModuleId,
	traits::AccountIdConversion
};

use wika_traits::OwnershipRegistry ;



/// Configure the pallet by specifying the parameters and types on which it depends.
/// Reminder: this Trait will be implemented by the Runtime to include this pallet.
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId> ;
	type MaxLengthURL: Get<u8> ;
	type MaxLikes: Get<u8> ;
	type OwnershipRegistry: OwnershipRegistry<Self> ;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance ;


fn u128_to_balance<T:Config>(input: u128) -> BalanceOf<T> {
	input.saturated_into()
}

fn num_likes_to_price<T:Config>(num_likes: u8) -> u128 {
	let num_likes_u128: u128 = num_likes.into() ;
	LikePrice::get() * num_likes_u128
}

fn num_likes_to_balance<T:Config>(num_likes: u8) -> BalanceOf<T> {
	let price_u128: u128 = num_likes_to_price::<T>(num_likes) ;
	u128_to_balance::<T>(price_u128)
}

const PALLET_ID: ModuleId = ModuleId(*b"LIKE_ME!");



// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where
    		AccountId = <T as frame_system::Config>::AccountId {
        /// Event emitted when a like is processed. [who, url]
        Liked(AccountId, Vec<u8>),
    }
}

// Errors inform users that something went wrong.
decl_error! {
    pub enum Error for Module<T: Config> {
        /// Not enough balance to like.
        NotEnoughBalanceToLike,
        /// Too many likes.
        TooManyLikes,
        /// Sender already in queue.
        AlreadyInQueue,
        /// URL is too long.
        UrlTooLong
    }
}

// The pallet's runtime storage items.
// https://substrate.dev/docs/en/knowledgebase/runtime/storage
decl_storage! {
    trait Store for Module<T: Config> as Likes {
    	UrlCount: u128 = 0 ;
    	LikePrice: u128 = 1_000_000_000_000;
    	Urls: map hasher(blake2_128_concat) Vec<u8> => (u64, T::AccountId, T::AccountId) ;
        Likes: double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) Vec<u8> => (u64, u8, u8, T::AccountId) ;
    }
}


// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
impl<T: Config> Module<T> {

	fn pot_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

	pub fn pot() -> BalanceOf<T> {
		T::Currency::free_balance(&Self::pot_id())
	}

	fn pay_recipients(sender: &T::AccountId, url: &Vec<u8>, first_recipient: T::AccountId, num_likes: u8)
		-> (T::AccountId, u8) {
		debug::debug!(target: "LIKE", "Paying the target recipients: {:?}", num_likes);
		let mut recipient = first_recipient ;
		let mut remaining_likes: u8 = num_likes ;
		while (remaining_likes>0) && (recipient!=Self::pot_id())  {
			let recipient_data = Likes::<T>::take(&recipient, &url) ;
			let recipient_likes = recipient_data.2 ;
			debug::debug!(target: "LIKE", "While loop head...");
			debug::debug!(target: "LIKE", "recicient: {:?}", recipient);
			debug::debug!(target: "LIKE", "recipient_likes: {:?}", recipient_likes);
			debug::debug!(target: "LIKE", "remaining_likes: {:?}", remaining_likes);
			if remaining_likes >= recipient_likes {
				// Enough likes to pay this recipient in full
				debug::debug!(target: "LIKE", "CASE A - FullPayment") ;
				debug::debug!(target: "LIKE", "Preparing to transfer from: {:?}", &sender);
				debug::debug!(target: "LIKE", "to: {:?}", recipient);
				debug::debug!(target: "LIKE", "balance: {:?}", num_likes_to_balance::<T>(recipient_likes));
				T::Currency::transfer(&sender,
									  &recipient,
									  num_likes_to_balance::<T>(recipient_likes),
									  ExistenceRequirement::KeepAlive).expect("balance was already checked") ;
				remaining_likes -= recipient_likes ;
				// Done with recipient re-inserting with zero balance
				let recipient_data_update = (recipient_data.0, recipient_data.1, 0, recipient_data.3.clone()) ;
				Likes::<T>::insert(&recipient, &url, recipient_data_update) ;
				debug::debug!(target: "LIKE", "Recipient data updated: {:?}", &recipient);
				// Moving to next recipient in line
				recipient = recipient_data.3 ;
			} else {
				// Partial payment for this recipient
				debug::debug!(target: "LIKE", "CASE B - PartialPayment") ;
				debug::debug!(target: "LIKE", "Preparing to transfer from: {:?}", &sender);
				debug::debug!(target: "LIKE", "to: {:?}", &recipient);
				debug::debug!(target: "LIKE", "balance: {:?}", num_likes_to_balance::<T>(remaining_likes));
				T::Currency::transfer(&sender,
									  &recipient,
									  num_likes_to_balance::<T>(remaining_likes),
									  ExistenceRequirement::KeepAlive).expect("balance was already checked") ;
				let recipient_likes_update = recipient_likes - remaining_likes ;
				// Update this recipient state
				let recipient_data_update = (recipient_data.0, recipient_data.1, recipient_likes_update, recipient_data.3) ;
				Likes::<T>::insert(&recipient, &url, recipient_data_update) ;
				debug::debug!(target: "LIKE", "Recipient data updated: {:?}", &recipient);
				// Done with payments, will exit the loop with zero remaining likes
				remaining_likes = 0 ;
			}
		}
		(recipient, remaining_likes)
	}

	fn send_to_pot(sender: &T::AccountId, num_likes: u8) {
		debug::debug!(target: "LIKE", "Sending likes to pot: {:?}", num_likes);
		T::Currency::transfer(sender,
							  &Self::pot_id(),
							  num_likes_to_balance::<T>(num_likes),
							  ExistenceRequirement::KeepAlive).expect("balance was already checked");
	}

	fn add_to_chain(sender: &T::AccountId, url: &Vec<u8>,
							 first_in_line: T::AccountId, last_in_line: T::AccountId) {
		let account = if last_in_line==Self::pot_id() {
			first_in_line
		} else {
			last_in_line
		} ;
		debug::debug!(target: "LIKE", "Adding sender to the chain: {:?}", &sender) ;
		debug::debug!(target: "LIKE", "Previous account: {:?}", &account) ;
		let data = Likes::<T>::take(&account, &url) ;
		let update = (data.0, data.1, data.2, sender.clone()) ;
		Likes::<T>::insert(&account, &url, update) ;
	}

	fn update_url(sender: &T::AccountId, url: &Vec<u8>, current_num: u64, num_likes: u8, next_recipient: T::AccountId) {
		let num_likes_u64: u64 = num_likes.into() ;
		let num_likes_update: u64 = current_num+num_likes_u64 ;
		if next_recipient==Self::pot_id() {
			// If we reached pot_id it means we cleared all recipients from this URL
			// Sender becomes first in line
			debug::debug!(target: "LIKE", "Sender is becoming first in line: {:?}", &sender);
			let new_url_data = (num_likes_update, &sender, &Self::pot_id()) ;
			Urls::<T>::insert(&url, new_url_data);
		} else {
			// Otherwise update first in line and put sender as last
			debug::debug!(target: "LIKE", "Updating first in line and sender") ;
			let new_url_data = (num_likes_update, &next_recipient, &sender) ;
			Urls::<T>::insert(&url, new_url_data);

		}
		debug::debug!(target: "LIKE", "url state updated: {:?}", &url);
	}

	fn like_existing_url(sender: &T::AccountId, url: &Vec<u8>, num_likes: u8) {
		// Take URL data
		debug::debug!(target: "LIKE", "Like existing url: {:?}", &url);
		let data = Urls::<T>::take(&url) ;
		let current_total_likes = data.0 ;
		// Start by creating the Like record for this sender...
		debug::debug!(target: "LIKE", "Creating like record: {:?}", &sender);
		Likes::<T>::insert(&sender, &url, (current_total_likes, num_likes, num_likes*2, Self::pot_id())) ;
		// Pay the target recipients
		let (next_recipient, remaining_likes) = Self::pay_recipients(&sender, url, data.1.clone(), num_likes) ;
		// Send additional likes to the pot
		if remaining_likes>0 {
			Self::send_to_pot(&sender, remaining_likes) ;
		}
		// Add this sender in the queue chain
		Self::add_to_chain(&sender, &url, data.1, data.2) ;
		// Update the Url state
		Self::update_url(&sender, &url, data.0, num_likes, next_recipient) ;
	}

	fn like_new_url(sender: &T::AccountId, url: &Vec<u8>, num_likes: u8) {
		// Create the URL record for the first time
		// and send the likes to the pot
		debug::debug!(target: "LIKE", "Creating url state for first time: {:?}", &url);
		let total_likes:u64 = num_likes.into() ;
		Urls::<T>::insert(&url, (total_likes, &sender, &Self::pot_id()));
		debug::debug!(target: "LIKE", "Creating first like record: {:?}", &sender);
		Likes::<T>::insert(&sender, &url, (0, num_likes, num_likes*2, &Self::pot_id()));
		debug::debug!(target: "LIKE", "Sending likes to pot: {:?}", num_likes);
		T::Currency::transfer(&sender,
							  &Self::pot_id(),
							  num_likes_to_balance::<T>(num_likes),
							  ExistenceRequirement::KeepAlive).expect("balance was already checked");
		// Update total count of Urls
		let url_count = UrlCount::take() + 1 ;
		UrlCount::set(url_count) ;
		debug::debug!(target: "LIKE", "Updated url_count: {:?}", url_count);
	}

}

decl_module! {

    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        // Errors must be initialized if they are used by the pallet.
        type Error = Error<T>;

        // Events must be initialized if they are used by the pallet.
        fn deposit_event() = default;

        /// Create a new question
        #[weight = 10_000]
        fn like(origin, url: Vec<u8>, num_likes: u8) {
            // Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;

			// Check that there's enough funds to pay for the likes
			let total_price_u128 = num_likes_to_price::<T>(num_likes) ;
			let total_price_balance = u128_to_balance::<T>(total_price_u128) ;
			let free = T::Currency::free_balance(&sender) ;
			ensure!(free>total_price_balance, Error::<T>::NotEnoughBalanceToLike) ;

			// Check that num_likes is smaller than MaxLikes
			ensure!(num_likes<=T::MaxLikes::get(), Error::<T>::TooManyLikes) ;

			// Check that URL is not too long
			ensure!(url.len()<T::MaxLengthURL::get().into(), Error::<T>::UrlTooLong) ;

            // Store the new like.
            if Urls::<T>::contains_key(&url) {
            	// Check that the sender is not already in line
				ensure!(!Likes::<T>::contains_key(&sender, &url), Error::<T>::AlreadyInQueue);
				// Go for all changes
            	&Self::like_existing_url(&sender, &url, num_likes) ;
            } else {
            	&Self::like_new_url(&sender, &url, num_likes) ;
            }

            // Emit an event that the like was processed.
            Self::deposit_event(RawEvent::Liked(sender, url));
        }

    }
}
