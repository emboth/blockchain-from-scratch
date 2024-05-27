//! Now that we have a functioning hash-linked data structure, we can use it to actually
//! track some state. Here we will start to explore the idea of extrinsics and state by
//! slightly abusing the header's extrinsics_root and state_root fields. As the names imply,
//! these are typically used for Merkle roots of large data sets. But in our case we will use
//! these fields to directly contain a single extrinsic per block, and a single piece of state.
//!
//! In the coming parts of this tutorial, we will expand this to be more real-world like and
//! use some real batching.

use crate::hash;

// We will use Rust's built-in hashing where the output type is u64. I'll make an alias
// so the code is slightly more readable.
type Hash = u64;

/// The header is now expanded to contain an extrinsic and a state. Note that we are not
/// using roots yet, but rather directly embedding some minimal extrinsic and state info
/// into the header.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Header {
    parent: Hash,
    height: u64,
    extrinsic: u64,
    state: u64,
    // Still no consensus. That's the next part.
    consensus_digest: (),
}

// Here are the methods for creating new header and verifying headers.
// It is your job to write them.
impl Header {
    /// Returns a new valid genesis header.
    fn genesis() -> Self {
        Header {
            parent: 0,
            height: 0,
            extrinsic: 0,
            state: 0,
            consensus_digest: (),
        }
    }

    /// Create and return a valid child header.
    fn child(&self, extrinsic: u64) -> Self {
        let new_state = self.state.wrapping_add(extrinsic); // Example state update logic

        // Create the new header
        Header {
            parent: hash(self),
            height: self.height + 1,
            extrinsic,
            state: new_state,
            consensus_digest: (),
        }
    }

    /// Verify that all the given headers form a valid chain from this header to the tip.
    ///
    /// In addition to the consecutive heights and linked hashes, we now need to consider our state.
    /// This blockchain will work as an adder. That means that the state starts at zero,
    /// and at each block we add the extrinsic to the state.
    ///
    /// So in order for a block to verify, we must have that relationship between the extrinsic,
    /// the previous state, and the current state.
    fn verify_sub_chain(&self, chain: &[Header]) -> bool {
        let mut previous_header = self;

        for header in chain {
            // Check height
            if header.height != previous_header.height + 1 {
                return false;
            }

            // Check parent hash
            if header.parent != hash(previous_header) {
                return false;
            }

            // Check state update
            if header.state != previous_header.state.wrapping_add(header.extrinsic) {
                //handle potential overflow
                return false;
            }

            previous_header = header;
        }

        true
    }
}

// And finally a few functions to use the code we just

/// Build and return a valid chain with the given number of blocks.
fn build_valid_chain(n: u64) -> Vec<Header> {
    let mut headers = vec![Header::genesis()];

    // Iterate to create each subsequent header
    for i in 1..n {
        // Create a child header with some example extrinsic value (e.g., i)
        let child = headers.last().unwrap().child(i);
        headers.push(child);
    }

    headers
}

/// Build and return a chain with at least three headers.
/// The chain should start with a proper genesis header,
/// but the entire chain should NOT be valid.
///
/// As we saw in the last unit, this is trivial when we construct arbitrary blocks.
/// However, from outside this crate, it is not so trivial. Our interface for creating
/// new blocks, `genesis()` and `child()`, makes it impossible to create arbitrary blocks.
///
/// For this function, ONLY USE the the `genesis()` and `child()` methods to create blocks.
/// The exercise is still possible.
fn build_an_invalid_chain() -> Vec<Header> {
    // Start with a valid genesis header
    let mut headers = vec![Header::genesis()];

    // Create the first child, which will be valid
    let child1 = headers.last().unwrap().child(1);
    headers.push(child1);

    // Create the second child, which will also be valid
    let child2 = headers.last().unwrap().child(2);
    headers.push(child2);

    // Create an invalid third child by using the first header as the parent, breaking the chain
    let invalid_child = headers[0].child(3);
    headers.push(invalid_child);

    headers
}

/// Build and return two header chains.
/// Both chains should individually be valid.
/// They should have the same genesis header.
/// They should not be the exact same chain.
///
/// Here is an example of two such chains:
///            /-- 3 -- 4
/// G -- 1 -- 2
///            \-- 3'-- 4'
///
/// Side question: What is the fewest number of headers you could create to achieve this goal.
fn build_forked_chain() -> (Vec<Header>, Vec<Header>) {
    // Start with a valid genesis header
    let genesis = Header::genesis();

    // Create the first child header from the genesis
    let child1 = genesis.child(1);

    // Create the second child header from the first child
    let child2 = child1.child(2);

    // Create the first fork from the second child
    let fork1_child3 = child2.child(3);
    let fork1_child4 = fork1_child3.child(4);

    // Create the second fork from the second child
    let fork2_child3 = child2.child(5); // Different extrinsic to differentiate the chain
    let fork2_child4 = fork2_child3.child(6);

    // Construct the two chains
    let chain1 = vec![
        genesis.clone(),
        child1.clone(),
        child2.clone(),
        fork1_child3,
        fork1_child4,
    ];
    let chain2 = vec![genesis, child1, child2, fork2_child3, fork2_child4];

    (chain1, chain2)

    // Exercise 7: After you have completed this task, look at how its test is written below.
    // There is a critical thinking question for you there.
}

// To run these tests: `cargo test bc_2`
#[test]
fn bc_2_genesis_block_height() {
    let g = Header::genesis();
    assert!(g.height == 0);
}

#[test]
fn bc_2_genesis_block_parent() {
    let g = Header::genesis();
    assert!(g.parent == 0);
}

#[test]
fn bc_2_genesis_block_extrinsic() {
    // Typically genesis blocks do not have any extrinsics.
    // In Substrate they never do. So our convention is to have the extrinsic be 0.
    let g = Header::genesis();
    assert!(g.extrinsic == 0);
}

#[test]
fn bc_2_genesis_block_state() {
    let g = Header::genesis();
    assert!(g.state == 0);
}

#[test]
fn bc_2_child_block_height() {
    let g = Header::genesis();
    let b1 = g.child(0);
    assert!(b1.height == 1);
}

#[test]
fn bc_2_child_block_parent() {
    let g = Header::genesis();
    let b1 = g.child(0);
    assert!(b1.parent == hash(&g));
}

#[test]
fn bc_2_child_block_extrinsic() {
    let g = Header::genesis();
    let b1 = g.child(7);
    assert_eq!(b1.extrinsic, 7);
}

#[test]
fn bc_2_child_block_state() {
    let g = Header::genesis();
    let b1 = g.child(7);
    assert_eq!(b1.state, 7);
}

#[test]
fn bc_2_verify_genesis_only() {
    let g = Header::genesis();

    assert!(g.verify_sub_chain(&[]));
}

#[test]
fn bc_2_verify_three_blocks() {
    let g = Header::genesis();
    let b1 = g.child(5);
    let b2 = b1.child(6);

    assert_eq!(b2.state, 11);
    assert!(g.verify_sub_chain(&[b1, b2]));
}

#[test]
fn bc_2_cant_verify_invalid_parent() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.parent = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_cant_verify_invalid_number() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.height = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_cant_verify_invalid_state() {
    let g = Header::genesis();
    let mut b1 = g.child(5);
    b1.state = 10;

    assert!(!g.verify_sub_chain(&[b1]));
}

#[test]
fn bc_2_invalid_chain_is_really_invalid() {
    // This test chooses to use the student's own verify function.
    // This should be relatively safe given that we have already tested that function.
    let invalid_chain = build_an_invalid_chain();
    assert!(!invalid_chain[0].verify_sub_chain(&invalid_chain[1..]))
}

#[test]
fn bc_2_verify_forked_chain() {
    let g = Header::genesis();
    let (c1, c2) = build_forked_chain();

    // Both chains have the same valid genesis block
    assert_eq!(g, c1[0]);
    assert_eq!(g, c2[0]);

    // Both chains are individually valid
    assert!(g.verify_sub_chain(&c1[1..]));
    assert!(g.verify_sub_chain(&c2[1..]));

    // The two chains are not identical
    // Question for students: I've only compared the last blocks here.
    // Is that enough? Is it possible that the two chains have the same final block,
    // but differ somewhere else?
    assert_ne!(c1.last(), c2.last());
}
