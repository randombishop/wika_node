#![cfg_attr(not(feature = "std"), no_std)]


pub trait OwnershipRegistry {
    fn get_owner() ;
}
