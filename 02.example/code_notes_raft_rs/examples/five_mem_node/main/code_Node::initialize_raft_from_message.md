#1.Node::initialize_raft_from_message

```
Node::initialize_raft_from_message
--let mut cfg = example_config();
--cfg.id = msg.to;
--let logger = logger.new(o!("tag" => format!("peer_{}", msg.to)));
--let storage = MemStorage::new();
--self.raft_group = Some(RawNode::new(&cfg, storage, &logger).unwrap());
```