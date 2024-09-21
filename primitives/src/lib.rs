#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::DispatchResult;
pub mod currency;

/// Signed version of Balance
pub type Amount = i128;

pub type Balance = u128;

pub trait Swap<Balance, AccountId> {
	fn get_target_amount(order_id: u32) -> Balance;

	fn inter_take_order(taker: AccountId, order_id: u32, receiver: AccountId) -> DispatchResult;
}
