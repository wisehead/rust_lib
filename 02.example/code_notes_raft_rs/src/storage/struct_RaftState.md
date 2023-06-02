#1.struct RaftState

```rust
/// Holds both the hard state (commit index, vote leader, term) and the configuration state
/// (Current node IDs)
#[derive(Debug, Clone, Default, Getters, Setters)]
pub struct RaftState {
    /// Contains the last meta information including commit index, the vote leader, and the vote term.
    pub hard_state: HardState,

    /// Records the current node IDs like `[1, 2, 3]` in the cluster. Every Raft node must have a
    /// unique ID in the cluster;
    pub conf_state: ConfState,
}

```