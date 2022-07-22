// https://github.com/nervosnetwork/merkle-tree/blob/master/src/merkle_tree.rs
// https://github.com/zruchala/merkletree/blob/main/src/main.rs
// https://github.com/zruchala/merkletree/blob/main/src/main.rs
// use crate::{collections::VecDeque, vec, vec::Vec};
use std::collections::VecDeque;
use std::vec;
use core::cmp::Reverse;
use core::marker::PhantomData;

pub trait Merge {
    type Item;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item;
}

pub struct MerkleTree<T, M> {
    nodes: Vec<T>,
    merge: PhantomData<M>,
}

impl<T, M> MerkleTree<T, M>
where
    T: Ord + Default + Clone,
    M: Merge<Item = T>,
{
    /// `leaf_indices`: The indices of leaves
    pub fn merkle_proof(&self, leaf_indices: &[u32]) -> Option<MerkleProof<T, M>> {
        if self.nodes.is_empty() || leaf_indices.is_empty() {
            return None;
        }

        let leaves_count = ((self.nodes.len() >> 1) + 1) as u32;
        let mut indices = leaf_indices
            .iter()
            .map(|i| leaves_count + i - 1)
            .collect::<Vec<_>>();

        indices.sort_by_key(|i| Reverse(*i));
        if indices[0] >= (leaves_count << 1) - 1 {
            return None;
        }

        let mut lemmas = Vec::new();
        let mut queue: VecDeque<u32> = indices.clone().into();

        while let Some(index) = queue.pop_front() {
            if index == 0 {
                assert!(queue.is_empty());
                break;
            }
            let sibling = index.sibling();
            if Some(&sibling) == queue.front() {
                queue.pop_front();
            } else {
                lemmas.push(self.nodes[sibling as usize].clone());
            }

            let parent = index.parent();
            if parent != 0 {
                queue.push_back(parent);
            }
        }

        indices.sort_by_key(|i| &self.nodes[*i as usize]);

        Some(MerkleProof {
            indices,
            lemmas,
            merge: PhantomData,
        })
    }

    pub fn merkle_root(&self) -> T {
        if self.nodes.is_empty() {
            T::default()
        } else {
            self.nodes[0].clone()
        }
    }

    pub fn nodes(&self) -> &[T] {
        &self.nodes
    }
}

pub struct MerkleProof<T, M> {
    indices: Vec<u32>,
    lemmas: Vec<T>,
    merge: PhantomData<M>,
}

impl<T, M> MerkleProof<T, M>
where
    T: Ord + Default + Clone,
    M: Merge<Item = T>,
{
    pub fn new(indices: Vec<u32>, lemmas: Vec<T>) -> Self {
        Self {
            indices,
            lemmas,
            merge: PhantomData,
        }
    }

    pub fn root(&self, leaves: &[T]) -> Option<T> {
        if leaves.len() != self.indices.len() || leaves.is_empty() {
            return None;
        }

        let mut leaves = leaves.to_vec();
        leaves.sort();

        let mut pre = self
            .indices
            .iter()
            .zip(leaves.into_iter())
            .map(|(i, l)| (*i, l))
            .collect::<Vec<_>>();
        pre.sort_by_key(|i| Reverse(i.0));

        let mut queue: VecDeque<(u32, T)> = pre.into();
        let mut lemmas_iter = self.lemmas.iter();

        while let Some((index, node)) = queue.pop_front() {
            if index == 0 {
                // ensure that all lemmas and leaves are consumed
                if lemmas_iter.next().is_none() && queue.is_empty() {
                    return Some(node);
                } else {
                    return None;
                }
            }

            if let Some(sibling) = match queue.front() {
                Some((front, _)) if *front == index.sibling() => queue.pop_front().map(|i| i.1),
                _ => lemmas_iter.next().cloned(),
            } {
                let parent_node = if index.is_left() {
                    M::merge(&node, &sibling)
                } else {
                    M::merge(&sibling, &node)
                };

                queue.push_back((index.parent(), parent_node));
            }
        }

        None
    }

    pub fn verify(&self, root: &T, leaves: &[T]) -> bool {
        match self.root(leaves) {
            Some(r) => &r == root,
            _ => false,
        }
    }

    pub fn indices(&self) -> &[u32] {
        &self.indices
    }

    pub fn lemmas(&self) -> &[T] {
        &self.lemmas
    }
}

#[derive(Default)]
pub struct CBMT<T, M> {
    data_type: PhantomData<T>,
    merge: PhantomData<M>,
}

impl<T, M> CBMT<T, M>
where
    T: Ord + Default + Clone,
    M: Merge<Item = T>,
{
    pub fn merkle_root(leaves: &[T]) -> T {
        if leaves.is_empty() {
            return T::default();
        }

        let mut queue = VecDeque::with_capacity((leaves.len() + 1) >> 1);

        let mut iter = leaves.rchunks_exact(2);
        while let Some([leaf1, leaf2]) = iter.next() {
            queue.push_back(M::merge(leaf1, leaf2))
        }
        if let [leaf] = iter.remainder() {
            queue.push_front(leaf.clone())
        }

        while queue.len() > 1 {
            let right = queue.pop_front().unwrap();
            let left = queue.pop_front().unwrap();
            queue.push_back(M::merge(&left, &right));
        }

        queue.pop_front().unwrap()
    }

    pub fn build_merkle_tree(leaves: &[T]) -> MerkleTree<T, M> {
        let len = leaves.len();
        if len > 0 {
            let mut nodes = vec![T::default(); len - 1];
            nodes.extend_from_slice(leaves);

            (0..len - 1)
                .rev()
                .for_each(|i| nodes[i] = M::merge(&nodes[(i << 1) + 1], &nodes[(i << 1) + 2]));

            MerkleTree {
                nodes,
                merge: PhantomData,
            }
        } else {
            MerkleTree {
                nodes: vec![],
                merge: PhantomData,
            }
        }
    }

    pub fn build_merkle_proof(leaves: &[T], leaf_indices: &[u32]) -> Option<MerkleProof<T, M>> {
        Self::build_merkle_tree(leaves).merkle_proof(leaf_indices)
    }
}

trait TreeIndex {
    fn sibling(&self) -> Self;
    fn parent(&self) -> Self;
    fn is_left(&self) -> bool;
}

macro_rules! impl_tree_index {
    ($t: ty) => {
        impl TreeIndex for $t {
            fn sibling(&self) -> $t {
                if *self == 0 {
                    0
                } else {
                    ((self + 1) ^ 1) - 1
                }
            }

            fn parent(&self) -> $t {
                if *self == 0 {
                    0
                } else {
                    (self - 1) >> 1
                }
            }

            fn is_left(&self) -> bool {
                self & 1 == 1
            }
        }
    };
}

impl_tree_index!(u32);
impl_tree_index!(usize);

#[cfg(test)]
mod tests {
    use super::*;
    
    use proptest::num::i32;

    struct MergeI32 {}

    impl Merge for MergeI32 {
        type Item = i32;
        fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
            right.wrapping_sub(*left)
        }
    }

    type CBMTI32 = CBMT<i32, MergeI32>;

    #[test]
    fn build_root_directly() {
        let leaves = vec![2i32, 3, 5, 7, 11];
        assert_eq!(4, CBMTI32::merkle_root(&leaves));
    }

    #[test]
    fn test_merkle_proof() {
        let leaves = vec![2i32, 3, 5, 7, 11, 13];
        let leaf_indices = vec![0u32, 5u32];
        let proof_leaves = leaf_indices
            .iter()
            .map(|i| leaves[*i as usize].clone())
            .collect::<Vec<_>>();
        let proof = CBMTI32::build_merkle_proof(&leaves, &leaf_indices).unwrap();

        assert_eq!(vec![11, 3, 2], proof.lemmas);
        assert_eq!(Some(1), proof.root(&proof_leaves));

        // merkle proof for single leaf
        let leaves = vec![2i32];
        let leaf_indices = vec![0u32];
        let proof_leaves = leaf_indices
            .iter()
            .map(|i| leaves[*i as usize].clone())
            .collect::<Vec<_>>();
        let proof = CBMTI32::build_merkle_proof(&leaves, &leaf_indices).unwrap();
        assert!(proof.lemmas.is_empty());
        assert_eq!(Some(2), proof.root(&proof_leaves));
    }
}
