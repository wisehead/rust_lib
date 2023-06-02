#1.struct Message

```rust
#[derive(PartialEq,Clone,Default)]
pub struct Message {
    // message fields
    pub msg_type: MessageType,
    pub to: u64,
    pub from: u64,
    pub term: u64,
    pub log_term: u64,
    pub index: u64,
    pub entries: ::protobuf::RepeatedField<Entry>,
    pub commit: u64,
    pub commit_term: u64,
    pub snapshot: ::protobuf::SingularPtrField<Snapshot>,
    pub request_snapshot: u64,
    pub reject: bool,
    pub reject_hint: u64,
    pub context: ::bytes::Bytes,
    pub deprecated_priority: u64,
    pub priority: i64,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}
```