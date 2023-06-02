#1.struct MemStorageCore

```rust

/// The Memory Storage Core instance holds the actual state of the storage struct. To access this
/// value, use the `rl` and `wl` functions on the main MemStorage implementation.
#[derive(Default)]
pub struct MemStorageCore {
    raft_state: RaftState,
    // entries[i] has raft log position i+snapshot.get_metadata().index
    entries: Vec<Entry>,
    // Metadata of the last snapshot received.
    snapshot_metadata: SnapshotMetadata,
    // If it is true, the next snapshot will return a
    // SnapshotTemporarilyUnavailable error.
    trigger_snap_unavailable: bool,
    // Peers that are fetching entries asynchronously.
    trigger_log_unavailable: bool,
    // Stores get entries context.
    get_entries_context: Option<GetEntriesContext>,
}
```