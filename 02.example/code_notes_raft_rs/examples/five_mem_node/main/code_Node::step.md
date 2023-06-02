#1.Node::step

```
Node::step
--if self.raft_group.is_none() {
----if is_initial_msg(&msg) {
------self.initialize_raft_from_message(&msg, logger);
--let raft_group = self.raft_group.as_mut().unwrap();
--let _ = raft_group.step(msg);
```