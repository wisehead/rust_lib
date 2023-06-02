#1.main

```
main
--let logger = slog::Logger::root(drain, o!());
--const NUM_NODES: u32 = 5;
--// Create 5 mailboxes to send/receive messages. Every node holds a `Receiver` to receive
--// messages from others, and uses the respective `Sender` to send messages to others.
--let (mut tx_vec, mut rx_vec) = (Vec::new(), Vec::new());
--for _ in 0..NUM_NODES {
        let (tx, rx) = mpsc::channel();
        tx_vec.push(tx);
        rx_vec.push(rx);
--let (tx_stop, rx_stop) = mpsc::channel();
--let rx_stop = Arc::new(Mutex::new(rx_stop));
--// A global pending proposals queue. New proposals will be pushed back into the queue, and
--// after it's committed by the raft cluster, it will be poped from the queue.
--let proposals = Arc::new(Mutex::new(VecDeque::<Proposal>::new()));
--let mut handles = Vec::new();
--for (i, rx) in rx_vec.into_iter().enumerate() {
----// A map[peer_id -> sender]. In the example we create 5 nodes, with ids in [1, 5].
----let mailboxes = (1..6u64).zip(tx_vec.iter().cloned()).collect();
----let mut node = match i {
------// Peer 1 is the leader.
------0 => Node::create_raft_leader(1, rx, mailboxes, &logger),
------// Other peers are followers.
------_ => Node::create_raft_follower(rx, mailboxes),
----let proposals = Arc::clone(&proposals);//每个raft node线程有独立的queue
----let rx_stop_clone = Arc::clone(&rx_stop);
----// Here we spawn the node on a new thread and keep a handle so we can join on them later.
----let handle = thread::spawn
------loop
--------thread::sleep(Duration::from_millis(10));
--------loop {
----------// Step raft messages.
----------match node.my_mailbox.try_recv() {
------------Ok(msg) => node.step(msg, &logger),
------------Err(TryRecvError::Empty) => break,
------------Err(TryRecvError::Disconnected) => return,
----------let raft_group = match node.raft_group {
------------Some(ref mut r) => r,
------------// When Node::raft_group is `None` it means the node is not initialized.
------------_ => continue,
----------if t.elapsed() >= Duration::from_millis(100) {
------------// Tick the raft.
------------raft_group.tick();
------------t = Instant::now();
----------// Let the leader pick pending proposals from the global queue.
----------if raft_group.raft.state == StateRole::Leader {
------------// Handle new proposals.
------------let mut proposals = proposals.lock().unwrap();
------------for p in proposals.iter_mut().skip_while(|p| p.proposed > 0) {
--------------propose(raft_group, p);

```