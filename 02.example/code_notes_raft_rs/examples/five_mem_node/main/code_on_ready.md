#1.on_ready

```
on_ready
--if !raft_group.has_ready() {
----return;
--let store = raft_group.raft.raft_log.store.clone();
--let mut ready = raft_group.ready();
```