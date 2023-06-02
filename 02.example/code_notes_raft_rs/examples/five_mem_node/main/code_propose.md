#1.propose

```
propose
--let last_index1 = raft_group.raft.raft_log.last_index() + 1;
--if let Some((ref key, ref value)) = proposal.normal {
----let data = format!("put {} {}", key, value).into_bytes();
----let _ = raft_group.propose(vec![], data);
--} else if let Some(ref cc) = proposal.conf_change {
----let _ = raft_group.propose_conf_change(vec![], cc.clone());
--} else if let Some(_transferee) = proposal.transfer_leader {
----// TODO: implement transfer leader.
----unimplemented!();
--let last_index2 = raft_group.raft.raft_log.last_index() + 1;
--if last_index2 == last_index1 {
        // Propose failed, don't forget to respond to the client.
----proposal.propose_success.send(false).unwrap();
--} else {
----proposal.proposed = last_index1;
```