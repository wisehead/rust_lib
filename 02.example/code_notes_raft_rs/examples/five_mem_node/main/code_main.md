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
```