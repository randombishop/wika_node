#![cfg_attr(not(feature = "std"), no_std)]

use sp_std::prelude::* ;
use frame_support::{
    decl_module, decl_storage, decl_event, decl_error,
	ensure, StorageMap,
	PalletId,
	traits::{Currency, ExistenceRequirement, Get}
};
use frame_system::ensure_signed;
use sp_std::vec::Vec;
use sp_runtime::{
	SaturatedConversion,
	traits::AccountIdConversion
};

use wika_traits::OwnershipRegistry ;



/// Configure the pallet by specifying the parameters and types on which it depends.
/// Reminder: this Trait will be implemented by the Runtime to include this pallet.
pub trait Config: frame_system::Config {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
	type Currency: Currency<Self::AccountId> ;
	type MaxLengthURL: Get<u8> ;
	type OwnershipRegistry: OwnershipRegistry<Self> ;
}

type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance ;


fn u128_to_balance<T:Config>(input: u128) -> BalanceOf<T> {
	input.saturated_into()
}

fn num_likes_to_price<T:Config>(num_likes: u32) -> u128 {
	let num_likes_u128: u128 = num_likes.into() ;
	LikePrice::get() * num_likes_u128
}

fn num_likes_to_balance<T:Config>(num_likes: u32, share: u8) -> BalanceOf<T> {
	let price_u128: u128 = num_likes_to_price::<T>(num_likes) ;
	let share_u128: u128 = share.into() ;
	let value = price_u128 * share_u128 / 100 ;
	return u128_to_balance::<T>(value) ;
}

const PALLET_ID: PalletId = PalletId(*b"LIKE_ME!");



// Pallets use events to inform users when important changes are made.
// Event documentation should end with an array that provides descriptive names for parameters.
// https://substrate.dev/docs/en/knowledgebase/runtime/events
decl_event! {
    pub enum Event<T> where
    		AccountId = <T as frame_system::Config>::AccountId {
        /// Event emitted when a like is processed. [who, url]
        Liked(AccountId, Vec<u8>, u32),
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

    	// Number of URLs liked so far
    	UrlCount: u128 = 0 ;

    	// Price to submit 1 Like
    	LikePrice: u128 = 1_000_000_000_000;

    	// Maximum number of likes allowed
    	MaxLikes: u32 = 100 ;

    	// How likes are split amongst authors, referrers and previous likers
    	ShareAuthor: u8 = 33 ;
    	ShareReferrer: u8 = 33 ;
    	SharePreviousLikers: u8 = 33 ;

    	// Number of times users will keep receiving rewards once they enter the line
    	NumRoundsToRewardLikers: u8 = 4 ;

    	// URL likes
    	// - u64: Number of likes received by this URL.
    	// - AccountId: Current liker waiting in line to receive their rewards
    	// - AccountId: Last liker in line who will receive rewards
    	//              (this will be used to update the chain when next one comes in.)
    	Urls: map hasher(blake2_128_concat) Vec<u8> => (u64, T::AccountId, T::AccountId) ;

    	// Like records by URL / USER
    	// - u64: Number of previous likes at the URL when the user submitted his.
    	// - u32: Number of likes
    	// - u32: Number of rounds during which payback will be received
    	// - AccountId: Next liker in line
        Likes: double_map hasher(blake2_128_concat) T::AccountId, hasher(blake2_128_concat) Vec<u8> => (u64, u32, u32, T::AccountId) ;

    }
}


// Dispatchable functions allows users to interact with the pallet and invoke state changes.
// These functions materialize as "extrinsics", which are often compared to transactions.
// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
impl<T: Config> Module<T> {

	fn get_pot_id() -> T::AccountId {
        PALLET_ID.into_account()
    }

	fn _get_pot_balance() -> BalanceOf<T> {
		T::Currency::free_balance(&Self::get_pot_id())
	}

	fn pay(sender: &T::AccountId, num_likes: u32, recipient: &T::AccountId, share: u8) {
		let amount = num_likes_to_balance::<T>(num_likes, share) ;
		log::debug!(target: "LIKE", "paying {:?} from {:?} to {:?}", &amount, &sender, &recipient);
		T::Currency::transfer(sender,
							  recipient,
							  amount,
							  ExistenceRequirement::KeepAlive).expect("balance was already checked");
	}

	fn pay_previous_liker(sender: &T::AccountId, num_likes: u32, recipient: &T::AccountId) {
		Self::pay(sender, num_likes, recipient, SharePreviousLikers::get()) ;
	}

	fn pay_author(sender: &T::AccountId, num_likes: u32, recipient: &T::AccountId) {
		Self::pay(sender, num_likes, recipient, ShareAuthor::get()) ;
	}

	fn pay_referrer(sender: &T::AccountId, num_likes: u32, recipient: &T::AccountId) {
		Self::pay(sender, num_likes, recipient, ShareReferrer::get()) ;
	}

	fn pay_extra(sender: &T::AccountId, num_likes: u32, recipient: &T::AccountId) {
		let share = 100 - (SharePreviousLikers::get() + ShareAuthor::get() + ShareReferrer::get()) ;
		Self::pay(sender, num_likes, recipient, share) ;
	}

	fn pay_author_referrer_and_extra(sender: &T::AccountId, url: &Vec<u8>, url_ref: &Vec<u8>, num_likes: u32) {
		// Pay author
		let author = T::OwnershipRegistry::get_owner(url) ;
		Self::pay_author(&sender, num_likes, &author) ;

		// Pay referrer
		let referrer = T::OwnershipRegistry::get_owner(url_ref) ;
		Self::pay_referrer(&sender, num_likes, &referrer) ;

		// Send extra tip to the pot
		let pot = Self::get_pot_id() ;
		Self::pay_extra(&sender, num_likes, &pot) ;
	}

	fn pay_previous_likers(sender: &T::AccountId, url: &Vec<u8>, first_recipient: T::AccountId, num_likes: u32)
		-> (T::AccountId, u32) {
		log::debug!(target: "LIKE", "Paying the target recipients: {:?}", num_likes);
		let mut recipient = first_recipient ;
		let mut remaining_likes: u32 = num_likes ;
		while (remaining_likes>0) && (recipient!=Self::get_pot_id())  {
			let recipient_data = Likes::<T>::take(&recipient, &url) ;
			let recipient_likes = recipient_data.2 ;
			log::debug!(target: "LIKE", "While loop head...");
			log::debug!(target: "LIKE", "recicient: {:?}", recipient);
			log::debug!(target: "LIKE", "recipient_likes: {:?}", recipient_likes);
			log::debug!(target: "LIKE", "remaining_likes: {:?}", remaining_likes);
			if remaining_likes >= recipient_likes {
				// Enough likes to pay this recipient in full
				log::debug!(target: "LIKE", "CASE A - FullPayment") ;
				log::debug!(target: "LIKE", "Preparing to transfer from: {:?}", &sender);
				log::debug!(target: "LIKE", "to: {:?}", recipient);
				log::debug!(target: "LIKE", "recipient_likes: {:?}", recipient_likes);
				Self::pay_previous_liker(&sender, recipient_likes, &recipient) ;
				remaining_likes -= recipient_likes ;
				// Done with recipient re-inserting with zero balance
				let recipient_data_update = (recipient_data.0, recipient_data.1, 0, recipient_data.3.clone()) ;
				Likes::<T>::insert(&recipient, &url, recipient_data_update) ;
				log::debug!(target: "LIKE", "Recipient data updated: {:?}", &recipient);
				// Moving to next recipient in line
				recipient = recipient_data.3 ;
			} else {
				// Partial payment for this recipient
				log::debug!(target: "LIKE", "CASE B - PartialPayment") ;
				log::debug!(target: "LIKE", "Preparing to transfer from: {:?}", &sender);
				log::debug!(target: "LIKE", "to: {:?}", &recipient);
				log::debug!(target: "LIKE", "remaining_likes: {:?}", remaining_likes);
				Self::pay_previous_liker(&sender, remaining_likes, &recipient) ;
				let recipient_likes_update = recipient_likes - remaining_likes ;
				// Update this recipient state
				let recipient_data_update = (recipient_data.0, recipient_data.1, recipient_likes_update, recipient_data.3) ;
				Likes::<T>::insert(&recipient, &url, recipient_data_update) ;
				log::debug!(target: "LIKE", "Recipient data updated: {:?}", &recipient);
				// Done with payments, will exit the loop with zero remaining likes
				remaining_likes = 0 ;
			}
		}
		(recipient, remaining_likes)
	}



	fn add_to_chain(sender: &T::AccountId, url: &Vec<u8>,
							 first_in_line: T::AccountId, last_in_line: T::AccountId) {
		let account = if last_in_line==Self::get_pot_id() {
			first_in_line
		} else {
			last_in_line
		} ;
		log::debug!(target: "LIKE", "Adding sender to the chain: {:?}", &sender) ;
		log::debug!(target: "LIKE", "Previous account: {:?}", &account) ;
		let data = Likes::<T>::take(&account, &url) ;
		let update = (data.0, data.1, data.2, sender.clone()) ;
		Likes::<T>::insert(&account, &url, update) ;
	}

	fn update_url(sender: &T::AccountId, url: &Vec<u8>, current_num: u64, num_likes: u32, next_recipient: T::AccountId) {
		let num_likes_u64: u64 = num_likes.into() ;
		let num_likes_update: u64 = current_num+num_likes_u64 ;
		if next_recipient==Self::get_pot_id() {
			// If we reached pot_id it means we cleared all recipients from this URL
			// Sender becomes first in line
			log::debug!(target: "LIKE", "Sender is becoming first in line: {:?}", &sender);
			let new_url_data = (num_likes_update, &sender, &Self::get_pot_id()) ;
			Urls::<T>::insert(&url, new_url_data);
		} else {
			// Otherwise update first in line and put sender as last
			log::debug!(target: "LIKE", "Updating first in line and sender") ;
			let new_url_data = (num_likes_update, &next_recipient, &sender) ;
			Urls::<T>::insert(&url, new_url_data);

		}
		log::debug!(target: "LIKE", "url state updated: {:?}", &url);
	}

	fn like_existing_url(sender: &T::AccountId, url: &Vec<u8>, ref_url: &Vec<u8>, num_likes: u32) {

		// Take URL data
		log::debug!(target: "LIKE", "like_existing_url url: {:?}", &url);
		let data = Urls::<T>::take(&url) ;
		let current_total_likes = data.0 ;

		// Start by creating the Like record for this sender...
		log::debug!(target: "LIKE", "like_existing_url Creating like record: {:?}", &sender);
		let rounds: u32 = NumRoundsToRewardLikers::get().into() ;
		Likes::<T>::insert(&sender, &url, (current_total_likes, num_likes, num_likes*rounds, Self::get_pot_id())) ;

		// Pay the previous likers
		log::debug!(target: "LIKE", "like_existing_url pay_previous_likers: {:?}", &num_likes);
		let (next_recipient, remaining_likes) = Self::pay_previous_likers(&sender, url, data.1.clone(), num_likes) ;
		log::debug!(target: "LIKE", "like_existing_url next_recipient: {:?}", &next_recipient);
		log::debug!(target: "LIKE", "like_existing_url remaining_likes: {:?}", &remaining_likes);

		// Distribute additional likes
		if remaining_likes>0 {
			let previous_liker = Self::get_pot_id() ;
			Self::pay_previous_liker(&sender, remaining_likes, &previous_liker) ;
		}

		// Pay author, referrer and extra
		Self::pay_author_referrer_and_extra(&sender, url, ref_url, num_likes) ;

		// Add this sender in the queue chain
		Self::add_to_chain(&sender, &url, data.1, data.2) ;

		// Update the Url state
		Self::update_url(&sender, &url, data.0, num_likes, next_recipient) ;
	}

	fn like_new_url(sender: &T::AccountId, url: &Vec<u8>, url_ref: &Vec<u8>, num_likes: u32) {
		// Create the URL record for the first time
		log::debug!(target: "LIKE", "Creating url state for first time: {:?}", &url);
		let total_likes:u64 = num_likes.into() ;
		Urls::<T>::insert(&url, (total_likes, &sender, Self::get_pot_id()));
		log::debug!(target: "LIKE", "Creating first like record: {:?}", &sender);
		let rounds: u32 = NumRoundsToRewardLikers::get().into() ;
		Likes::<T>::insert(&sender, &url, (0, num_likes, num_likes*rounds, Self::get_pot_id()));
		log::debug!(target: "LIKE", "Sending likes to pot: {:?}", num_likes);

		// The share of previous likers goes to the pot
		let previous_liker = Self::get_pot_id() ;
		Self::pay_previous_liker(&sender, num_likes, &previous_liker) ;

		// Pay author, referrer and extra
		Self::pay_author_referrer_and_extra(&sender, url, url_ref, num_likes) ;

		// Update total count of Urls
		let url_count = UrlCount::take() + 1 ;
		UrlCount::set(url_count) ;
		log::debug!(target: "LIKE", "Updated url_count: {:?}", url_count);
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
        fn like(origin, url: Vec<u8>, url_ref: Vec<u8>, num_likes: u32) {
            // Check that the extrinsic was signed and get the signer.
            let sender = ensure_signed(origin)?;

			// Check that there's enough funds to pay for the likes
			let total_price_u128 = num_likes_to_price::<T>(num_likes) ;
			let total_price_balance = u128_to_balance::<T>(total_price_u128) ;
			let free = T::Currency::free_balance(&sender) ;
			ensure!(free>total_price_balance, Error::<T>::NotEnoughBalanceToLike) ;

			// Check that num_likes is smaller than MaxLikes
			let max_likes: u32 = MaxLikes::get() ;
			ensure!(num_likes<=max_likes, Error::<T>::TooManyLikes) ;

			// Check that URL is not too long
			ensure!(url.len()<T::MaxLengthURL::get().into(), Error::<T>::UrlTooLong) ;

            // Store the new like.
            if Urls::<T>::contains_key(&url) {
            	ensure!(!Likes::<T>::contains_key(&sender, &url), Error::<T>::AlreadyInQueue);
            	Self::like_existing_url(&sender, &url, &url_ref, num_likes) ;
            } else {
            	Self::like_new_url(&sender, &url, &url_ref, num_likes) ;
            }

            // Emit an event that the like was processed.
            Self::deposit_event(RawEvent::Liked(sender, url, num_likes));
        }

    }
}
