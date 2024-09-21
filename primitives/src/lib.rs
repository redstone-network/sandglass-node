#![cfg_attr(not(feature = "std"), no_std)]

pub mod currency;

/// Signed version of Balance
pub type Amount = i128;

pub type Balance = u128;
