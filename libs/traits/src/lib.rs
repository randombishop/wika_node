#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::Config ;
use sp_std::vec::Vec;

pub trait OwnershipRegistry<T:Config> {

    fn get_pot_id() -> T::AccountId ;

    fn get_owner(url: &Vec<u8>) -> T::AccountId ;

}


pub trait AuthorityRegistry<T:Config> {

    fn list_grandpa() ;

}
