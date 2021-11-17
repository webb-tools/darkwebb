// This file is part of Webb.

// Copyright (C) 2021 Webb Technologies Inc.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! # Anchor Module
//!
//! A simple module for building Anchors.
//!
//! ## Overview
//!
//! The Anchor module provides functionality for the following:
//!
//! * Inserting elements to the tree
//!
//! The supported dispatchable functions are documented in the [`Call`] enum.
//!
//! ### Terminology
//!
//! ### Goals
//!
//! The Anchor system in Webb is designed to make the following possible:
//!
//! * Define.
//!
//! ## Interface
//!
//! ## Related Modules
//!
//! * [`System`](../frame_system/index.html)
//! * [`Support`](../frame_support/index.html)

// Ensure we're `no_std` when compiling for Wasm.
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::type_complexity, clippy::too_many_arguments)]
#[cfg(test)]
pub mod mock;
#[cfg(test)]
mod test_utils;
#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod zk_config;

pub mod types;
pub mod weights;
use codec::Encode;
use darkwebb_primitives::{
	linkable_tree::{LinkableTreeInspector, LinkableTreeInterface},
	vanchor::{VAnchorConfig, VAnchorInspector, VAnchorInterface},
	verifier::*,
};
use frame_support::{dispatch::DispatchResult, ensure, pallet_prelude::DispatchError, traits::Get};
use orml_traits::MultiCurrency;
use sp_runtime::traits::AccountIdConversion;
use sp_std::prelude::*;
use types::*;
pub use weights::WeightInfo;

/// Type alias for the orml_traits::MultiCurrency::Balance type
pub type BalanceOf<T, I> =
	<<T as Config<I>>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::Balance;
/// Type alias for the orml_traits::MultiCurrency::CurrencyId type
pub type CurrencyIdOf<T, I> =
	<<T as pallet::Config<I>>::Currency as MultiCurrency<<T as frame_system::Config>::AccountId>>::CurrencyId;

pub use pallet::*;
#[frame_support::pallet]
pub mod pallet {
	use super::*;
	use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*, PalletId};
	use frame_system::pallet_prelude::*;

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T, I = ()>(_);

	#[pallet::config]
	/// The module configuration trait.
	pub trait Config<I: 'static = ()>: frame_system::Config + pallet_linkable_tree::Config<I> {
		/// The overarching event type.
		type Event: From<Event<Self, I>> + IsType<<Self as frame_system::Config>::Event>;

		#[pallet::constant]
		type PalletId: Get<PalletId>;

		/// The tree type
		type LinkableTree: LinkableTreeInterface<pallet_linkable_tree::LinkableTreeConfigration<Self, I>>
			+ LinkableTreeInspector<pallet_linkable_tree::LinkableTreeConfigration<Self, I>>;

		/// The verifier
		type Verifier: VerifierModule;

		/// Currency type for taking deposits
		type Currency: MultiCurrency<Self::AccountId>;

		type PostDepositHook: PostDepositHook<Self, I>;

		/// Native currency id
		#[pallet::constant]
		type NativeCurrencyId: Get<CurrencyIdOf<Self, I>>;

		/// Weight info for pallet
		type WeightInfo: WeightInfo;
	}

	/// The map of trees to their anchor metadata
	#[pallet::storage]
	#[pallet::getter(fn anchors)]
	pub type Anchors<T: Config<I>, I: 'static = ()> = StorageMap<
		_,
		Blake2_128Concat,
		T::TreeId,
		Option<AnchorMetadata<T::AccountId, BalanceOf<T, I>, CurrencyIdOf<T, I>>>,
		ValueQuery,
	>;

	/// The map of trees to their spent nullifier hashes
	#[pallet::storage]
	#[pallet::getter(fn nullifier_hashes)]
	pub type NullifierHashes<T: Config<I>, I: 'static = ()> =
		StorageDoubleMap<_, Blake2_128Concat, T::TreeId, Blake2_128Concat, T::Element, bool, ValueQuery>;

	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config<I>, I: 'static = ()> {
		/// New tree created
		AnchorCreation { tree_id: T::TreeId },
		/// Amount has been withdrawn from the anchor
		Withdraw { who: T::AccountId, amount: BalanceOf<T, I> },
		/// Amount has been deposited into the anchor
		Deposit {
			depositor: T::AccountId,
			tree_id: T::TreeId,
			leaf: T::Element,
			amount: BalanceOf<T, I>,
		},
		/// Post deposit hook has executed successfully
		PostDeposit {
			depositor: T::AccountId,
			tree_id: T::TreeId,
			leaf: T::Element,
		},
	}

	#[pallet::error]
	pub enum Error<T, I = ()> {
		/// Invalid Merkle Roots
		InvalidMerkleRoots,
		/// Unknown root
		UnknownRoot,
		/// Invalid withdraw proof
		InvalidWithdrawProof,
		/// Mixer not found.
		NoAnchorFound,
		/// Invalid nullifier that is already used
		/// (this error is returned when a nullifier is used twice)
		AlreadyRevealedNullifier,
	}

	#[pallet::hooks]
	impl<T: Config<I>, I: 'static> Hooks<BlockNumberFor<T>> for Pallet<T, I> {}

	#[pallet::call]
	impl<T: Config<I>, I: 'static> Pallet<T, I> {
		#[pallet::weight(<T as Config<I>>::WeightInfo::create(*depth as u32, *max_edges))]
		pub fn create(
			origin: OriginFor<T>,
			max_edges: u32,
			depth: u8,
			asset: CurrencyIdOf<T, I>,
		) -> DispatchResultWithPostInfo {
			// Should it only be the root who can create anchors?
			ensure_root(origin)?;
			let tree_id = <Self as VAnchorInterface<_>>::create(T::AccountId::default(), depth, max_edges, asset)?;
			Self::deposit_event(Event::AnchorCreation { tree_id });
			Ok(().into())
		}

		#[pallet::weight(0)] // TODO: Fix after benchmarks
		pub fn transact(
			origin: OriginFor<T>,
			id: T::TreeId,
			proof_bytes: Vec<u8>,
			public_amount: BalanceOf<T, I>,
			ext_amount: BalanceOf<T, I>,
			ext_data_hash: T::Element,
			input_nullifiers: Vec<T::Element>,
			output_commitments: Vec<T::Element>,
			roots: Vec<T::Element>,
			recipient: T::AccountId,
			relayer: T::AccountId,
			fee: BalanceOf<T, I>,
		) -> DispatchResultWithPostInfo {
			ensure_signed(origin)?;
			<Self as VAnchorInterface<_>>::transact(
				id,
				&proof_bytes,
				public_amount,
				ext_amount,
				ext_data_hash,
				input_nullifiers,
				output_commitments,
				roots,
				recipient,
				relayer,
				fee,
			)?;
			Ok(().into())
		}
	}
}

pub struct VAnchorConfigration<T: Config<I>, I: 'static>(core::marker::PhantomData<T>, core::marker::PhantomData<I>);

impl<T: Config<I>, I: 'static> VAnchorConfig for VAnchorConfigration<T, I> {
	type AccountId = T::AccountId;
	type Balance = BalanceOf<T, I>;
	type ChainId = T::ChainId;
	type CurrencyId = CurrencyIdOf<T, I>;
	type Element = T::Element;
	type LeafIndex = T::LeafIndex;
	type TreeId = T::TreeId;
}

impl<T: Config<I>, I: 'static> VAnchorInterface<VAnchorConfigration<T, I>> for Pallet<T, I> {
	fn create(
		creator: T::AccountId,
		depth: u8,
		max_edges: u32,
		asset: CurrencyIdOf<T, I>,
	) -> Result<T::TreeId, DispatchError> {
		let id = T::LinkableTree::create(creator.clone(), max_edges, depth)?;
		Ok(id)
	}

	fn transact(
		id: T::TreeId,
		proof_bytes: &[u8],
		public_amount: BalanceOf<T, I>,
		ext_amount: BalanceOf<T, I>,
		ext_data_hash: T::Element,
		input_nullifiers: Vec<T::Element>,
		output_commitments: Vec<T::Element>,
		roots: Vec<T::Element>,
		recipient: T::AccountId,
		relayer: T::AccountId,
		fee: BalanceOf<T, I>,
	) -> Result<(), DispatchError> {
		// double check the number of roots
		T::LinkableTree::ensure_max_edges(id, roots.len())?;
		// Check if local root is known
		T::LinkableTree::ensure_known_root(id, roots[0])?;
		// Check if neighbor roots are known
		T::LinkableTree::ensure_known_neighbor_roots(id, &roots)?;

		// Check nullifier and add or return `InvalidNullifier`
		for nullifier in input_nullifiers {
			Self::ensure_nullifier_unused(id, nullifier)?;
		}

		// Format proof public inputs for verification
		// FIXME: This is for a specfic gadget so we ought to create a generic handler
		// FIXME: Such as a unpack/pack public inputs trait
		// FIXME: 	-> T::PublicInputTrait::validate(public_bytes: &[u8])
		//
		Ok(())
	}

	fn add_nullifier_hash(id: T::TreeId, nullifier_hash: T::Element) -> Result<(), DispatchError> {
		NullifierHashes::<T, I>::insert(id, nullifier_hash, true);
		Ok(())
	}

	fn add_edge(
		id: T::TreeId,
		src_chain_id: T::ChainId,
		root: T::Element,
		latest_leaf_index: T::LeafIndex,
	) -> Result<(), DispatchError> {
		T::LinkableTree::add_edge(id, src_chain_id, root, latest_leaf_index)
	}

	fn update_edge(
		id: T::TreeId,
		src_chain_id: T::ChainId,
		root: T::Element,
		latest_leaf_index: T::LeafIndex,
	) -> Result<(), DispatchError> {
		T::LinkableTree::update_edge(id, src_chain_id, root, latest_leaf_index)
	}
}

impl<T: Config<I>, I: 'static> VAnchorInspector<VAnchorConfigration<T, I>> for Pallet<T, I> {
	fn is_nullifier_used(tree_id: T::TreeId, nullifier_hash: T::Element) -> bool {
		NullifierHashes::<T, I>::contains_key(tree_id, nullifier_hash)
	}

	fn ensure_nullifier_unused(id: T::TreeId, nullifier: T::Element) -> Result<(), DispatchError> {
		ensure!(
			!Self::is_nullifier_used(id, nullifier),
			Error::<T, I>::AlreadyRevealedNullifier
		);
		Ok(())
	}

	fn has_edge(id: T::TreeId, src_chain_id: T::ChainId) -> bool {
		T::LinkableTree::has_edge(id, src_chain_id)
	}
}

impl<T: Config<I>, I: 'static> Pallet<T, I> {
	pub fn account_id() -> T::AccountId {
		T::PalletId::get().into_account()
	}

	pub fn get_anchor(
		id: T::TreeId,
	) -> Result<AnchorMetadata<T::AccountId, BalanceOf<T, I>, CurrencyIdOf<T, I>>, DispatchError> {
		let anchor = Anchors::<T, I>::get(id);
		ensure!(anchor.is_some(), Error::<T, I>::NoAnchorFound);
		Ok(anchor.unwrap())
	}
}

pub trait PostDepositHook<T: Config<I>, I: 'static> {
	fn post_deposit(depositor: T::AccountId, id: T::TreeId, leaf: T::Element) -> DispatchResult;
}

impl<T: Config<I>, I: 'static> PostDepositHook<T, I> for () {
	fn post_deposit(_: T::AccountId, _: T::TreeId, _: T::Element) -> DispatchResult {
		Ok(())
	}
}
/// Truncate and pad 256 bit slice
pub fn truncate_and_pad(t: &[u8]) -> Vec<u8> {
	let mut truncated_bytes = t[..20].to_vec();
	truncated_bytes.extend_from_slice(&[0u8; 12]);
	truncated_bytes
}