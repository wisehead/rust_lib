#1.struct RawNode

```rust
/// RawNode is a thread-unsafe Node.
/// The methods of this struct correspond to the methods of Node and are described
/// more fully there.
pub struct RawNode<T: Storage> {
    /// The internal raft state.
    pub raft: Raft<T>,
    prev_ss: SoftState,
    prev_hs: HardState,
    // Current max number of Record and ReadyRecord.
    max_number: u64,
    records: VecDeque<ReadyRecord>,
    // Index which the given committed entries should start from.
    commit_since_index: u64,
}

```