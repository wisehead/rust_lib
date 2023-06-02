#1.struct ReadyRecord

```rust
/// ReadyRecord encapsulates some needed data from the corresponding Ready.
#[derive(Default, Debug, PartialEq)]
struct ReadyRecord {
    number: u64,
    // (index, term) of the last entry from the entries in Ready
    last_entry: Option<(u64, u64)>,
    // (index, term) of the snapshot in Ready
    snapshot: Option<(u64, u64)>,
}
```