use primitive_types::H256;
use sha3::{Keccak256, Digest};
use evm_core::ExitError;


pub trait HasHash: Sized {  // Add Sized to allow Self in constructor
    fn hash(&self) -> H256;  // Get the hash value
    fn from_hash(hash: H256) -> Self;  // Constructor from hash
}

impl HasHash for H256 {
    fn hash(&self) -> H256 {
        *self  // H256 is already a hash, so we just return itself
    }

    fn from_hash(hash: H256) -> Self {
        hash  // Similarly, we just return the hash directly
    }
}

// T is the type of data stored in the leaf nodes
#[derive(Clone, Debug)]
pub struct MerkleNode<T> {
    pub data: T,
}

impl<T> MerkleNode<T> {
    pub fn new(data: T) -> Self {
        Self {
            data
        }
    }
    
    pub fn new_empty() -> Self 
    where T: Default 
    {
        Self {
            data: T::default(),
        }
    }
}

pub fn hasher() -> Keccak256 {
    Keccak256::new()
}

#[derive(Clone, Debug)]
pub struct MerkleTree<T: HasHash> {
	nodes: Vec<MerkleNode<T>>,
	left_most_leaf: usize,
	depth: usize,
}

impl<T: Clone + HasHash> MerkleTree<T> {
    const DEFAULT_DEPTH: usize = 20;

    pub fn default() -> Self 
    where T: Default 
    {
        Self::new(Self::DEFAULT_DEPTH)
    }

    pub fn new(depth: usize) -> Self 
    where T: Default
    {
        if depth == 0 {
            panic!("Merkle tree depth must be greater than 0");
        }
        Self {
            depth,
            left_most_leaf: 1 << depth,  // First leaf position
            nodes: vec![MerkleNode::<T>::new_empty(); (1 << (depth + 1)) + 24],
        }
    }

    fn compress(&self, l: &MerkleNode<T>, r: &MerkleNode<T>) -> MerkleNode<T> {
        let mut hasher = hasher();
        hasher.update(l.data.hash().as_bytes());
        hasher.update(r.data.hash().as_bytes());
        
        let hash = hasher.finalize();
        MerkleNode::<T>::new(T::from_hash(H256::from_slice(&hash)))
    }

    pub fn insert(&mut self, data: T) -> Result<(), ExitError> {
        if self.left_most_leaf >= (1 << (self.depth + 1)) {
            return Err(ExitError::MerkleTreeFull);
        }

        self.nodes[self.left_most_leaf] = MerkleNode::new(data);
        let mut i = self.left_most_leaf;
        while i > 1 {
            if i % 2 == 0 {
                self.nodes[i/2] = self.compress(&self.nodes[i], &self.nodes[i+1]);
            }
            else {
                self.nodes[i/2] = self.compress(&self.nodes[i-1], &self.nodes[i]);
            }
            i = i / 2;
        }
        self.left_most_leaf += 1;
        Ok(())
    }

}

// Fixed recursive Default implementation
impl<T: Clone + HasHash + Default> Default for MerkleTree<T> {
    fn default() -> Self {
        Self::new(Self::DEFAULT_DEPTH)
    }
}
