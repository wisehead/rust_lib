#1.Node::create_raft_leader

```
Node::create_raft_leader
--cfg.id = id;
--let logger = logger.new(o!("tag" => format!("peer_{}", id)));
--let mut s = Snapshot::default();
// Because we don't use the same configuration to initialize every node, so we use
        // a non-zero index to force new followers catch up logs by snapshot first, which will
        // bring all nodes to the same initial state.
--s.mut_metadata().index = 1;
--s.mut_metadata().term = 1;
--s.mut_metadata().mut_conf_state().voters = vec![1];
--let storage = MemStorage::new();
--storage.wl().apply_snapshot(s).unwrap();
--let raft_group = Some(RawNode::new(&cfg, storage, &logger).unwrap());
--Node {
            raft_group,
            my_mailbox,
            mailboxes,
            kv_pairs: Default::default(),
        }
```