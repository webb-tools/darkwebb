//! All the traits exposed to be used in other custom pallets
use frame_support::dispatch;
use sp_std::vec::Vec;

pub trait LinkableTreeConfig {
	type BlockNumber;
	type AccountId;
	type ChainId;
	type TreeId;
	type Element;
}

/// LinkableTree trait definition to be used in other pallets
pub trait LinkableTreeInterface<C: LinkableTreeConfig> {
	// Creates a new linkable tree
	fn create(creator: C::AccountId, depth: u8, max_edges: u32) -> Result<C::TreeId, dispatch::DispatchError>;
	/// Add an edge to this tree
	fn add_edge(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		root: C::Element,
		height: C::BlockNumber,
	) -> Result<(), dispatch::DispatchError>;
	/// Update an edge for this tree
	fn update_edge(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		root: C::Element,
		height: C::BlockNumber,
	) -> Result<(), dispatch::DispatchError>;
}

/// Trait for inspecting tree state
pub trait LinkableTreeInspector<C: LinkableTreeConfig> {
	/// Gets the merkle root for a tree or returns `TreeDoesntExist`
	fn get_neighbor_roots(id: C::TreeId) -> Result<Vec<C::Element>, dispatch::DispatchError>;
	/// Checks if a merkle root is in a tree's cached history or returns
	/// `TreeDoesntExist
	fn is_known_neighbor_root(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		target: C::Element,
	) -> Result<bool, dispatch::DispatchError>;

	// let is_known = Self::is_known_neighbor_root(id, src_chain_id, target)?;
	// ensure!(is_known, Error::<T, I>::InvalidNeighborWithdrawRoot);
	// Ok(())
	fn ensure_known_neighbor_root(
		id: C::TreeId,
		src_chain_id: C::ChainId,
		target: C::Element,
	) -> Result<(), dispatch::DispatchError>;
	/// Check if this anchor has this edge
	fn has_edge(id: C::TreeId, src_chain_id: C::ChainId) -> bool;
}