#![cfg_attr(not(feature = "std"), no_std)]

use frame_system::Config ;
use sp_std::vec::Vec;
use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use pallet_grandpa::AuthorityId as GrandpaId;

pub trait OwnershipRegistry<T:Config> {

    fn get_pot_id() -> T::AccountId ;

    fn get_owner(url: &Vec<u8>) -> T::AccountId ;

}


pub trait AuthorityRegistry<T:Config> {

    fn list_aura() -> Vec<AuraId> ;

    fn list_grandpa() -> Vec<GrandpaId> ;

}
