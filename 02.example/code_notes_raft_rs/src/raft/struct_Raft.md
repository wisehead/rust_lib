#1.struct Raft

```rust
/// A struct that represents the raft consensus itself. Stores details concerning the current
/// and possible state the system can take.
pub struct Raft<T: Storage> {
    prs: ProgressTracker,

    /// The list of messages.
    pub msgs: Vec<Message>,
    /// Internal raftCore.
    pub r: RaftCore<T>,
}

```