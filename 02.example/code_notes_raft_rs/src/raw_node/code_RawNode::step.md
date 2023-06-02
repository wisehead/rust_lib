#1.RawNode::step

```
RawNode::step
--if is_local_msg(m.get_msg_type()) {
----return Err(Error::StepLocalMsg);
--if self.raft.prs().get(m.from).is_some() || !is_response_msg(m.get_msg_type()) {
----return self.raft.step(m);
```