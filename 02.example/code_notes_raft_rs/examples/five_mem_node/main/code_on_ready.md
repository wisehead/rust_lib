#1.on_ready

```
on_ready
--if !raft_group.has_ready() {
----return;
--let store = raft_group.raft.raft_log.store.clone();
--let mut ready = raft_group.ready();
--let handle_messages = |msgs: Vec<Message>| {
        for msg in msgs {
            let to = msg.to;
            if mailboxes[&to].send(msg).is_err() {
                error!(
                    logger,
                    "send raft message to {} fail, let Raft retry it", to
                );
            }
        }
    };
--if !ready.messages().is_empty() {
    // Send out the messages come from the node.
----handle_messages(ready.take_messages());
--// Apply the snapshot. It's necessary because in `RawNode::advance` we stabilize the snapshot.
--if *ready.snapshot() != Snapshot::default() {
----let s = ready.snapshot().clone();
----if let Err(e) = store.wl().apply_snapshot(s) {

--//lambda
--let mut handle_committed_entries =
        |rn: &mut RawNode<MemStorage>, committed_entries: Vec<Entry>| {
----for entry in committed_entries {
------if let EntryType::EntryConfChange = entry.get_entry_type() {
--------//
------else {
        // For normal proposals, extract the key-value pair and then
        // insert them into the kv engine.
--------let data = str::from_utf8(&entry.data).unwrap();
--------let reg = Regex::new("put ([0-9]+) (.+)").unwrap();
--------if let Some(caps) = reg.captures(data) {
----------kv_pairs.insert(caps[1].parse().unwrap(), caps[2].to_string());
------if rn.raft.state == StateRole::Leader {
        // The leader should response to the clients, tell them if their proposals
        // succeeded or not.
--------let proposal = proposals.lock().unwrap().pop_front().unwrap();
--------proposal.propose_success.send(true).unwrap();

--// Apply all committed entries.
--handle_committed_entries(raft_group, ready.take_committed_entries());

--// Persistent raft logs. It's necessary because in `RawNode::advance` we stabilize
--// raft logs to the latest position.
--if let Err(e) = store.wl().append(ready.entries()) {

--if let Some(hs) = ready.hs() {
    // Raft HardState changed, and we need to persist it.
----store.wl().set_hardstate(hs.clone());
--if !ready.persisted_messages().is_empty() {
    // Send out the persisted messages come from the node.
----handle_messages(ready.take_persisted_messages());
--// Call `RawNode::advance` interface to update position flags in the raft.
--let mut light_rd = raft_group.advance(ready);
```