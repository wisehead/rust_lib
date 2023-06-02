#1.struct ConfState

```rust
#[derive(PartialEq,Clone,Default)]
pub struct ConfState {
    // message fields
    pub voters: ::std::vec::Vec<u64>,
    pub learners: ::std::vec::Vec<u64>,
    pub voters_outgoing: ::std::vec::Vec<u64>,
    pub learners_next: ::std::vec::Vec<u64>,
    pub auto_leave: bool,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

```