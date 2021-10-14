//! Autogenerated weights for `pallet_mt`
//!
//! THIS FILE WAS AUTO-GENERATED USING THE SUBSTRATE BENCHMARK CLI VERSION 4.0.0-dev
//! DATE: 2021-10-14, STEPS: `50`, REPEAT: 20, LOW RANGE: `[]`, HIGH RANGE: `[]`
//! EXECUTION: Some(Wasm), WASM-EXECUTION: Compiled, CHAIN: Some("dev"), DB CACHE: 128

// Executed Command:
// ./target/release/darkwebb-standalone-node
// benchmark
// --chain=dev
// --execution
// wasm
// --wasm-execution
// compiled
// --pallet
// pallet_mt
// --extrinsic
// *
// --steps
// 50
// --repeat
// 20
// --raw
// --output
// ./pallets/mt/weights.rs


#![cfg_attr(rustfmt, rustfmt_skip)]
#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

pub trait WeightInfo {
	fn create(d: u32) -> Weight;
	fn insert() -> Weight;
	fn set_maintainer() -> Weight;
	fn force_set_maintainer() -> Weight;
	fn force_set_default_hashes(p: u32) -> Weight;
}

/// Weight functions for `pallet_mt`.
pub struct WebbWeight<T>(PhantomData<T>);
impl<T: frame_system::Config> WeightInfo for WebbWeight<T> {
	// Storage: MerkleTree NextTreeId (r:1 w:1)
	// Storage: MerkleTree DefaultHashes (r:1 w:0)
	// Storage: MerkleTree Trees (r:0 w:1)
	fn create(_d: u32, ) -> Weight {
		(51_144_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(2 as Weight))
			.saturating_add(T::DbWeight::get().writes(2 as Weight))
	}
	// Storage: MerkleTree Trees (r:1 w:1)
	// Storage: MerkleTree NextLeafIndex (r:1 w:1)
	// Storage: MerkleTree DefaultHashes (r:1 w:0)
	// Storage: MerkleTree NextRootIndex (r:1 w:1)
	// Storage: MerkleTree Leaves (r:0 w:1)
	// Storage: MerkleTree CachedRoots (r:0 w:1)
	fn insert() -> Weight {
		(52_695_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(4 as Weight))
			.saturating_add(T::DbWeight::get().writes(5 as Weight))
	}
	// Storage: MerkleTree Maintainer (r:1 w:1)
	fn set_maintainer() -> Weight {
		(23_721_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: MerkleTree Maintainer (r:1 w:1)
	fn force_set_maintainer() -> Weight {
		(19_725_000 as Weight)
			.saturating_add(T::DbWeight::get().reads(1 as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
	// Storage: MerkleTree DefaultHashes (r:0 w:1)
	fn force_set_default_hashes(p: u32, ) -> Weight {
		(3_132_000 as Weight)
			// Standard Error: 0
			.saturating_add((62_000 as Weight).saturating_mul(p as Weight))
			.saturating_add(T::DbWeight::get().writes(1 as Weight))
	}
}

// For tests and backward compatibility
impl WeightInfo for () {
	fn create(d: u32) -> Weight {
		0
	}
	fn insert() -> Weight {
		0
	}
	fn set_maintainer() -> Weight{
		0
	}
	fn force_set_maintainer() -> Weight{
		0
	}
	fn force_set_default_hashes(p: u32) -> Weight{
		0
	}
}
