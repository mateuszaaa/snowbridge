// Copyright (C) 2020 Mangata team
// Based on Snowfork bridge implementation
//! # ETH
//!
//! An application that implements a bridged ETH asset.
//!
//! ## Overview
//!
//! ETH balances are stored in the tightly-coupled [`asset`] runtime module. When an account holder burns
//! some of their balance, a `Transfer` event is emitted. An external relayer will listen for this event
//! and relay it to the other chain.
//!
//! ## Interface
//!
//! This application implements the [`Application`] trait and conforms to its interface
//!
//! ### Dispatchable Calls
//!
//! - `burn`: Burn an ETH balance.
//!
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system::{self as system, ensure_signed};
use sp_core::{H160, U256};
use sp_std::prelude::*;

use artemis_asset as asset;
use artemis_core::{Application, BridgedAssetId};
use mangata_primitives::{Balance, TokenId};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_std::convert::TryInto;

mod payload;
use payload::Payload;

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

pub trait Trait: system::Trait + asset::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Erc20Module {

	}
}

decl_event!(
    /// Events for the ETH module.
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId
	{
		/// Signal a cross-chain transfer.
		Transfer(AccountId, H160, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// The submitted payload could not be decoded.
		InvalidPayload,
		/// Asset could not be burned
		BurnFailure,
		/// Passed amount is too big
		TooBigAmount,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		// Users should burn their holdings to release funds on the Ethereum side
		// TODO: Calculate weights
		#[weight = 0]
		pub fn burn(origin, recipient: H160, input_amount: U256) -> DispatchResult {
			let who = ensure_signed(origin)?;
			let asset_id = <asset::Module<T>>::get_native_asset_id(H160::zero());

			let amount: Balance = input_amount
				.try_into()
				.or(Err(Error::<T>::TooBigAmount))?;

			T::Currency::burn_and_settle(
				asset_id.into(),
				&who,
				amount.into(),
			).map_err(|_| Error::<T>::BurnFailure)?;

			Self::deposit_event(RawEvent::Transfer(who.clone(), recipient, input_amount));
			Ok(())
		}

			}
		}

impl<T: Trait> Module<T> {

	fn handle_event(payload: Payload<T::AccountId>) -> DispatchResult {
		let asset_id: BridgedAssetId = H160::zero();

		let amount: Balance = payload
			.amount
			.try_into()
			.or(Err(Error::<T>::TooBigAmount))?;

		if !<asset::Module<T>>::exists(asset_id) {
			let id: TokenId = T::Currency::create(&payload.recipient_addr, amount.into()).into();
			<asset::Module<T>>::link_assets(id, asset_id);
		} else {
			let id = <asset::Module<T>>::get_native_asset_id(asset_id);
			T::Currency::mint(id.into(), &payload.recipient_addr, amount.into())?;
		}

		Ok(())
	}
}

impl<T: Trait> Application for Module<T> {
	fn handle(payload: Vec<u8>) -> DispatchResult {
		let payload_decoded = Payload::decode(payload)
			.map_err(|_| Error::<T>::InvalidPayload)?;

		Self::handle_event(payload_decoded)
	}
}
