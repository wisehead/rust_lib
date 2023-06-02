#1.MemStorageCore::apply_snapshot

```
MemStorageCore::apply_snapshot
--let mut meta = snapshot.take_metadata();
--let index = meta.index;

--if self.first_index() > index {
----return Err(Error::Store(StorageError::SnapshotOutOfDate));

--self.snapshot_metadata = meta.clone();

--self.raft_state.hard_state.term = cmp::max(self.raft_state.hard_state.term, meta.term);
--self.raft_state.hard_state.commit = index;
--self.entries.clear();
--// Update conf states.
--self.raft_state.conf_state = meta.take_conf_state();
```