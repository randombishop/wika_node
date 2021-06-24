#![cfg_attr(not(feature = "std"), no_std)]



// Imports
// -------------------------------------------------

use frame_support::{
	ensure,
	decl_error, decl_event, decl_module, decl_storage
};

use frame_system::{
	ensure_root,
};

use wika_traits::AuthorityRegistry ;



// Pallet config
// -------------------------------------------------

pub trait Config: frame_system::Config   {
	type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
}





// Persistent data
// -------------------------------------------------

decl_storage! {
	trait Store for Module<T: Config> as Owners {
		// Total number of authorities registered
		AuthCount: u16 = 0 ;

    	// Registered authorities
    	// 0. Block at which they were registered
    	// 1. Enabled true/false
    	Authorities: map hasher(identity) T::AccountId => (T::BlockNumber, bool) ;
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
		AuthorityAdded(AccountId),
    	AuthorityEnabled(AccountId),
    	AuthorityDisabled(AccountId),
	}
);




// Errors
// -------------------------------------------------

decl_error! {
	pub enum Error for Module<T: Config> {

        // 0
        AuthorityAlreadyRegistered,

        // 1
        AuthorityNotRegistered,

        // 2
        InvalidAddress,

	}
}






// Implementation
// -------------------------------------------------


impl<T:Config> AuthorityRegistry<T> for Module<T> {

	fn list_grandpa() {

	}

}



impl<T: Config> Module<T> {

	fn is_registered(who: &T::AccountId) -> bool {
		Authorities::<T>::contains_key(who)
	}

	fn _is_enabled(who: &T::AccountId) -> bool {
		if Authorities::<T>::contains_key(who) {
			Authorities::<T>::get(who).1
		} else {
			false
		}
	}

}


decl_module! {
	pub struct Module<T: Config> for enum Call where origin: T::Origin {

		fn deposit_event() = default;

		// Add an authority
        #[weight = 10_000]
        fn add_authority(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is not already registered
			ensure!(!Self::is_registered(&account), Error::<T>::AuthorityAlreadyRegistered) ;

			// Add account as a new authority
			let current_block = <frame_system::Module<T>>::block_number();
			let authority = (current_block, true) ;
			Authorities::<T>::insert(&account, authority);

            // Emit an event that new validator was added.
            Self::deposit_event(RawEvent::AuthorityAdded(account));
        }

        // Disable an authority
        #[weight = 10_000]
        fn disable_authority(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is already registered
			ensure!(Self::is_registered(&account), Error::<T>::AuthorityNotRegistered) ;

			// Disable account
			let mut auth = Authorities::<T>::take(&account) ;
			auth.1 = false ;
			Authorities::<T>::insert(&account, &auth) ;

            // Emit an event that new validator was added.
            Self::deposit_event(RawEvent::AuthorityDisabled(account));
        }

        // Enable an authority
        #[weight = 10_000]
        fn enable_verifier(origin, account: T::AccountId) {
            // Check that the extrinsic is from sudo.
            ensure_root(origin)?;

			// Check that account is already in the list
			ensure!(Self::is_registered(&account), Error::<T>::AuthorityNotRegistered) ;

			// Enable account
			let mut auth = Authorities::<T>::take(&account) ;
			auth.1 = true ;
			Authorities::<T>::insert(&account, &auth) ;

            // Emit an event that verifier was enabled.
            Self::deposit_event(RawEvent::AuthorityEnabled(account));
        }

	}

}
