#1.struct HardState

```rust
#[derive(PartialEq,Clone,Default)]
pub struct HardState {
    // message fields
    pub term: u64,
    pub vote: u64,
    pub commit: u64,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

```