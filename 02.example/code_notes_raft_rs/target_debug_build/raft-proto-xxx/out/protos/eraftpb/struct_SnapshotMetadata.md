#1.struct SnapshotMetadata

```rust

#[derive(PartialEq,Clone,Default)]
pub struct SnapshotMetadata {
    // message fields
    pub conf_state: ::protobuf::SingularPtrField<ConfState>,
    pub index: u64,
    pub term: u64,
    // special fields
    pub unknown_fields: ::protobuf::UnknownFields,
    pub cached_size: ::protobuf::CachedSize,
}

```