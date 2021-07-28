//! # Asset
//!
//! The asset module provides functionality for handling bridged asset balances.
//!
//! ## Overview
//!
//! This module is used by the ETH and ERC20 pallets to store account balances for bridged assets.
//!
//! Each asset is identified by a unique `H160` hash. This is useful for tracking ERC20 tokens which on Ethereum are identified by a 20-byte contract address.
//!
//! For various reasons, we built our own asset pallet instead of reusing existing work:
//! - `assets`: Too high-level and limited for our needs
//! - `generic-asset`: Its enforced permissions system implies trusted operations. But our system is designed to be trustless.
//! - `balances`: Only stores balances for a single asset. Our ERC20 pallet supports multiple different ERC20 assets.
//!
//! Additionally, we need to store balances using `U256`, which seemed difficult or impossible to plug into the above pallets.
//!
//! ## Interface
//!
//! ### Dispatchable Calls
//!
//! - `transfer`: Transferring a balance between accounts.
//!
//! ### Public Functions
//!
//! - `do_mint`: Mint to an account's free balance.
//! - `do_burn`: Burn an account's free balance.
//!

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult};
use frame_system as system;
use mangata_primitives::{Balance, TokenId};
use orml_tokens::MultiTokenCurrencyExtended;
use sp_core::{RuntimeDebug, U256};
use sp_std::prelude::*;

use artemis_core::BridgedAssetId;

#[derive(Encode, Decode, Clone, PartialEq, Eq, Default, RuntimeDebug)]
pub struct AccountData {
	pub free: U256,
}


pub trait Trait: frame_system::Trait {
	type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
	type Currency: MultiTokenCurrencyExtended<Self::AccountId>;
}

decl_storage! {
	trait Store for Module<T: Trait> as Asset {
		pub NativeAsset get(fn get_native_asset_id): map hasher(blake2_128_concat) BridgedAssetId => TokenId;
		pub BridgedAsset get(fn get_bridged_asset_id): map hasher(blake2_128_concat) TokenId => BridgedAssetId;
	}
	add_extra_genesis {
		#[allow(clippy::type_complexity)]
		config(bridged_assets_links): Vec<(TokenId, BridgedAssetId, Balance, T::AccountId)>;
		build(|config: &GenesisConfig<T>|
		{
			for (native_asset_id, bridged_asset_id, initial_supply, initial_owner) in config.bridged_assets_links.iter(){
				let initialized_asset_id: TokenId = T::Currency::create(&initial_owner, {*initial_supply}.into()).into();
				assert!(initialized_asset_id == *native_asset_id, "Assets not initialized in the sequence of the asset ids provided");
				Module::<T>::link_assets(native_asset_id.to_owned(), bridged_asset_id.to_owned());

			}
		}
		)
	}
}

decl_event!(
	pub enum Event<T>
	where
		AccountId = <T as system::Trait>::AccountId,
	{
		Burned(BridgedAssetId, AccountId, U256),
		Minted(BridgedAssetId, AccountId, U256),
		Transferred(BridgedAssetId, AccountId, AccountId, U256),
	}
);

decl_error! {
	pub enum Error for Module<T: Trait> {
		/// Free balance got overflowed after transfer.
		FreeTransferOverflow,
		/// Total issuance got overflowed after minting.
		TotalMintingOverflow,
		/// Free balance got overflowed after minting.
		FreeMintingOverflow,
		/// Total issuance got underflowed after burning.
		TotalBurningUnderflow,
		/// Free balance got underflowed after burning.
		FreeBurningUnderflow,
		InsufficientBalance,
	}
}

decl_module! {

	pub struct Module<T: Trait> for enum Call where origin: T::Origin {

		type Error = Error<T>;

		fn deposit_event() = default;

		/// Transfer some free balance to another account.
		// TODO: Calculate weight
		#[weight = 0]
		pub fn transfer(origin, asset_id: BridgedAssetId, to: T::AccountId, amount: U256) -> DispatchResult {
			Ok(())
		}

	}
}

impl<T: Trait> Module<T> {
	pub fn link_assets(native_asset_id: TokenId, bridged_asset_id: BridgedAssetId) {
		NativeAsset::insert(bridged_asset_id, native_asset_id);
		BridgedAsset::insert(native_asset_id, bridged_asset_id);
	}

	pub fn exists(bridged_asset_id: BridgedAssetId) -> bool {
		NativeAsset::contains_key(bridged_asset_id)
	}
}
