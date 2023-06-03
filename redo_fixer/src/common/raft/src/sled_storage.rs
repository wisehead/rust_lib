//  Copyright 2022 Fabarta Authors.
//
//  Licensed under the Apache License, Version 2.0 (the "License");
//  you may not use this file except in compliance with the License.
//  You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
//  Unless required by applicable law or agreed to in writing, software
//  distributed under the License is distributed on an "AS IS" BASIS,
//  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//  See the License for the specific language governing permissions and
//  limitations under the License.

use crate::error::Result;
use crate::storage::LogStore;
use crate::utils::*;
use crate::Error;
use arrayref::array_ref;
use data_type::types::GraphId;
use data_type::types::PartitionId;
use data_type::types::TransactionId;
use data_type::types::RaftPeerId;
use data_type::ActionTypeLog;
use protobuf::Message;
use sled::{Batch, Db};
use std::cell::RefCell;
use std::cmp;
use std::collections::HashMap;
use tikv_raft::prelude::{ConfState, Entry, HardState, Snapshot, SnapshotMetadata};
use tikv_raft::RaftState;
use tikv_raft::Result as TiKvResult;
use tikv_raft::Storage;

const HARD_STATE: [u8; 8] = u64::MAX.to_le_bytes();
const CONF_STATE: [u8; 8] = (u64::MAX - 1).to_le_bytes();
const FIRST_INDEX: [u8; 8] = (u64::MAX - 2).to_le_bytes();
const LAST_INDEX: [u8; 8] = (u64::MAX - 3).to_le_bytes();

#[allow(dead_code)]
pub struct SledStorage {
    graph_id: u32,
    partition_id: u32,
    core: Db,
    active_txns: RefCell<HashMap<TransactionId, u64>>,
    snapshot_metadata: SnapshotMetadata,
}

unsafe impl Sync for SledStorage {}
unsafe impl Send for SledStorage {}

impl SledStorage {
    pub fn new(graph_id: GraphId, partition_id: PartitionId, peer_id: RaftPeerId, log_folder: Option<String>) -> Self {
        let file_path = if let Some(folder) = log_folder {
            format!("{folder}/{graph_id}_{partition_id}_{peer_id}")
        } else {
            format!("/tmp/raft_log/sled/{graph_id}_{partition_id}_{peer_id}")
        };
        Self {
            graph_id,
            partition_id,
            core: sled::open(file_path).unwrap(),
            active_txns: RefCell::default(),
            snapshot_metadata: SnapshotMetadata::default(),
        }
    }

    fn build_tikv_error(&self, e: Error) -> tikv_raft::Error {
        tikv_raft::Error::Store(tikv_raft::StorageError::Other(Box::new(e)))
    }

    fn get_entry(&self, index: u64) -> Result<Option<Entry>> {
        let bytes_entry = self.core.get(index.to_le_bytes())?;
        if let Some(bytes) = bytes_entry {
            let entry = Entry::parse_from_bytes(&bytes)?;
            Ok(Some(entry))
        } else {
            Ok(None)
        }
    }

    fn insert_hard_state(&self, hard_state: &HardState) -> Result<()> {
        let bytes = hard_state.write_to_bytes()?;
        self.core.insert(HARD_STATE, bytes)?;
        Ok(())
    }

    fn insert_conf_state(&self, conf_state: &ConfState) -> Result<()> {
        let bytes = conf_state.write_to_bytes()?;
        self.core.insert(CONF_STATE, bytes)?;
        Ok(())
    }

    fn get_minimum_index_active_txn(&self) -> Option<u64> {
        if self.active_txns.borrow().is_empty() {
            None
        } else {
            let mut r = u64::MAX;
            for entry_idx in self.active_txns.borrow().values() {
                if *entry_idx < r {
                    r = *entry_idx;
                }
            }
            Some(r)
        }
    }
}

impl LogStore for SledStorage {
    fn reset(&self) -> Result<()> {
        self.core.clear()?;
        Ok(())
    }

    fn get_checkpoint_idx(&self) -> Option<u64> {
        let hard_state = self.get_hard_state();
        if let Ok(Some(hs)) = hard_state {
            let active_txn_idx = self.get_minimum_index_active_txn();
            let commit_idx = hs.get_commit();
            if let Some(idx) = active_txn_idx {
                Some(cmp::min(idx, commit_idx))
            } else {
                Some(commit_idx)
            }
        } else {
            None
        }
    }

    fn get_first_index(&self) -> Result<u64> {
        let bytes_first_index = self.core.get(&FIRST_INDEX)?;
        if let Some(bytes) = bytes_first_index {
            let tmp = bytes.to_vec();
            Ok(u64::from_le_bytes(*array_ref![tmp.as_slice(), 0, 8]))
        } else {
            Ok(self.snapshot_metadata.index + 1)
        }
    }

    fn get_last_index(&self) -> Result<u64> {
        let bytes_last_index = self.core.get(&LAST_INDEX)?;
        if let Some(bytes) = bytes_last_index {
            let tmp = bytes.to_vec();
            Ok(u64::from_le_bytes(*array_ref![tmp.as_slice(), 0, 8]))
        } else {
            Ok(self.snapshot_metadata.index)
        }
    }

    fn get_term(&self, idx: u64) -> Result<u64> {
        let term = if let Some(entry) = self.get_entry(idx)? {
            entry.term
        } else {
            self.snapshot_metadata.term
        };
        arcgraph_log::raft_debug!(
            "get_term, idx={}, metadata.index={}, term={}",
            idx,
            self.snapshot_metadata.index,
            term
        );
        Ok(term)
    }

    fn get_entries(
        &self,
        low: u64,
        high: u64,
        max_size: impl Into<Option<u64>>,
    ) -> Result<Vec<Entry>> {
        let first_index = self.get_first_index()?;
        arcgraph_log::raft_debug!(
            "get_entries, low={}, high={}, first_index={}",
            low,
            high,
            first_index
        );
        if low < first_index {
            return Err(Error::Msg(String::from("Compacted")));
        }
        let last_index = self.get_last_index()?;
        if high > last_index + 1 {
            panic!(
                "index out of bound (last: {}, high: {})",
                last_index + 1,
                high
            );
        }

        let real_high = if let Some(size) = max_size.into() {
            if size < high - low {
                low + size + 1
            } else {
                high
            }
        } else {
            high
        };
        arcgraph_log::raft_debug!("get_entries, start={}, end={}", low, real_high);

        let start: &[u8] = &low.to_le_bytes();
        let end: &[u8] = &real_high.to_le_bytes();
        let r = self.core.range(start..end);
        let mut entries = vec![];
        for bytes_entry in r {
            let bytes = bytes_entry?;
            // entries.push(Entry::parse_from_bytes(&bytes.1).unwrap());
            if let Ok(entry) = Entry::parse_from_bytes(&bytes.1) {
                entries.push(entry);
            } else {
                panic!(
                    "entry parse from bytes error, bytes content is {:?}",
                    &bytes.1
                );
            }
        }
        arcgraph_log::raft_debug!(
            "get entries={:?}, low={:?}, high={:?}",
            entries,
            low,
            real_high
        );
        Ok(entries)
    }

    fn append(&mut self, entries: &[Entry]) -> Result<()> {
        arcgraph_log::raft_debug!("append={:?}", entries);
        if entries.is_empty() {
            Ok(())
        } else {
            let first_index = self.get_first_index()?;
            let last_index = self.get_last_index()?;
            arcgraph_log::raft_debug!(
                "append first_index={}, last_index={}",
                first_index,
                last_index
            );
            arcgraph_log::raft_debug!("append entries={:?}", entries);
            if first_index > entries[0].index {
                panic!(
                    "overwrite compacted raft logs, compacted: {}, append: {}",
                    first_index - 1,
                    entries[0].index,
                );
            }

            if last_index + 1 < entries[0].index {
                panic!(
                    "raft logs should be continuous, last index: {}, new appended: {}",
                    last_index, entries[0].index,
                );
            }

            self.core.transaction(|tx| {
                let mut insert_batch = Batch::default();
                for entry in entries.iter() {
                    let log_entries = convert_to_log_entries(entry);
                    for log_entry in log_entries.into_iter() {
                        if matches!(log_entry.action_type, ActionTypeLog::StartTxn) {
                            self.active_txns
                                .borrow_mut()
                                .insert(log_entry.txn_id, entry.index);
                        } else if matches!(
                            log_entry.action_type,
                            ActionTypeLog::CommitTxn | ActionTypeLog::RollbackTxn
                        ) {
                            self.active_txns.borrow_mut().remove(&log_entry.txn_id);
                        }
                        // else if matches!(log_entry.action_type, ActionTypeLog::SyncLSN){
                        //     let sled_util = SledUtil::instance();
                        //     let mut key = vec![];
                        //     key.extend_from_slice(self.graph_id.to_le_bytes().as_ref());
                        //     key.extend_from_slice(self.partition_id.to_le_bytes().as_ref());
                        //     let value = log_entry.lsn.to_be_bytes();
                        //     let _ = sled_util.insert(&key[..], sled::IVec::from(&value));
                        //     let _ = sled_util.flush();
                        // }
                    }
                    insert_batch.insert(
                        &entry.index.to_le_bytes(),
                        entry
                            .write_to_bytes()
                            .expect("entry should serialize to bytes"),
                    );
                }
                tx.apply_batch(&insert_batch)?;

                let mut del_batch = Batch::default();
                let del_last_idx = last_index;
                let del_start_idx = entries.last().unwrap().index + 1;
                arcgraph_log::raft_debug!(
                    "append del start={}, end={}",
                    del_start_idx,
                    del_last_idx
                );
                for key in del_start_idx..=del_last_idx {
                    del_batch.remove(&key.to_le_bytes());
                }
                tx.apply_batch(&del_batch)?;

                let new_last_idx = del_start_idx - 1;
                tx.insert(&LAST_INDEX, &new_last_idx.to_le_bytes())?;
                Ok(())
            })?;
            let last_index = self.get_last_index()?;
            arcgraph_log::raft_debug!(
                "first_index, last_index after append: {}, {}",
                first_index,
                last_index
            );
            Ok(())
        }
    }

    fn get_hard_state(&self) -> Result<Option<HardState>> {
        let bytes_hard_state = self.core.get(&HARD_STATE)?;
        if let Some(bytes) = bytes_hard_state {
            let hard_state = HardState::parse_from_bytes(&bytes)?;
            Ok(Some(hard_state))
        } else {
            Ok(None)
        }
    }

    fn get_conf_state(&self) -> Result<Option<ConfState>> {
        let bytes_conf_state = self.core.get(&CONF_STATE)?;
        if let Some(bytes) = bytes_conf_state {
            let conf_state = ConfState::parse_from_bytes(&bytes)?;
            Ok(Some(conf_state))
        } else {
            Ok(None)
        }
    }

    fn set_hard_state(&mut self, hard_state: &HardState) -> Result<()> {
        self.insert_hard_state(hard_state)
    }

    fn set_hard_state_commit(&mut self, commit: u64) -> Result<()> {
        let mut hard_state = self.get_hard_state()?.expect("hard state should exist");
        hard_state.set_commit(commit);
        self.insert_hard_state(&hard_state)
    }

    fn set_conf_state(&mut self, conf_state: &ConfState) -> Result<()> {
        self.insert_conf_state(conf_state)
    }

    fn create_snapshot(&mut self, _data: prost::bytes::Bytes) -> Result<Snapshot> {
        self.snapshot(0).map_err(|e| Error::Other(Box::new(e)))
    }

    fn apply_snapshot(&mut self, snapshot: Snapshot) -> Result<()> {
        let meta = snapshot.get_metadata();
        let first_index = self
            .get_first_index()
            .expect("storage should have first index");
        arcgraph_log::raft_debug!(
            "apply_snapshot, first_index={}, meta.index={}",
            first_index,
            meta.index
        );
        if first_index > meta.index {
            return Err(Error::Msg(String::from("Snapshot out of date")));
        }

        self.snapshot_metadata = meta.clone();
        let r = self.get_hard_state()?;
        let mut hard_state = if let Some(hs) = r {
            hs
        } else {
            HardState::default()
        };
        hard_state.term = cmp::max(meta.term, hard_state.term);
        hard_state.commit = cmp::max(meta.index, hard_state.commit);
        let conf_state = meta.get_conf_state();
        // arcgraph_log::raft_debug!("hard_state={:?}, conf_state={:?}", hard_state, conf_state);
        let last_index = self
            .get_last_index()
            .expect("storage should have last index");

        self.core.transaction(|tx| {
            let mut del_batch = Batch::default();
            for key in first_index..=last_index {
                del_batch.remove(&key.to_le_bytes());
            }
            tx.apply_batch(&del_batch)?;

            tx.insert(&FIRST_INDEX, &(meta.index + 1).to_le_bytes())?;
            tx.insert(&LAST_INDEX, &meta.index.to_le_bytes())?;

            tx.insert(
                &HARD_STATE,
                hard_state
                    .write_to_bytes()
                    .expect("hard state should be able to bytes"),
            )?;

            tx.insert(
                &CONF_STATE,
                conf_state
                    .write_to_bytes()
                    .expect("conf state should be able to bytes"),
            )?;
            Ok(())
        })?;
        self.active_txns.borrow_mut().clear();
        Ok(())
    }

    fn compact(&mut self, compact_idx: u64) -> Result<()> {
        let first_index = self.get_first_index()?;
        if compact_idx < first_index {
            return Ok(());
        }

        let last_index = self.get_last_index()?;
        if compact_idx > last_index + 1 {
            panic!(
                "compact not received raft logs: {}, last index: {}",
                compact_idx, last_index
            );
        }

        self.core.transaction(|tx| {
            let mut del_batch = Batch::default();
            for key in first_index..compact_idx {
                del_batch.remove(&key.to_le_bytes());
            }
            tx.apply_batch(&del_batch)?;
            tx.insert(&FIRST_INDEX, &compact_idx.to_le_bytes())?;
            Ok(())
        })?;
        Ok(())
    }
}

impl Storage for SledStorage {
    fn initial_state(&self) -> TiKvResult<RaftState> {
        let hard_state = if let Some(hs) = self
            .get_hard_state()
            .map_err(|e| self.build_tikv_error(e))?
        {
            hs
        } else {
            let tmp = HardState::default();
            self.insert_hard_state(&tmp)
                .map_err(|e| self.build_tikv_error(e))?;
            tmp
        };

        let conf_state = if let Some(cs) = self
            .get_conf_state()
            .map_err(|e| self.build_tikv_error(e))?
        {
            cs
        } else {
            let tmp = ConfState::default();
            self.insert_conf_state(&tmp)
                .map_err(|e| self.build_tikv_error(e))?;
            tmp
        };
        // arcgraph_log::raft_debug!("initial_state: {:?}, {:?}", hard_state, conf_state);
        Ok(RaftState {
            hard_state,
            conf_state,
        })
    }

    fn entries(
        &self,
        low: u64,
        high: u64,
        max_size: impl Into<Option<u64>>,
    ) -> TiKvResult<Vec<Entry>> {
        self.get_entries(low, high, max_size)
            .map_err(|e| self.build_tikv_error(e))
    }

    fn term(&self, idx: u64) -> TiKvResult<u64> {
        self.get_term(idx).map_err(|e| self.build_tikv_error(e))
    }

    fn first_index(&self) -> TiKvResult<u64> {
        self.get_first_index().map_err(|e| self.build_tikv_error(e))
    }

    fn last_index(&self) -> TiKvResult<u64> {
        self.get_last_index().map_err(|e| self.build_tikv_error(e))
    }

    fn snapshot(&self, request_index: u64) -> TiKvResult<Snapshot> {
        let mut snapshot = Snapshot::default();
        let meta = snapshot.mut_metadata();

        let hard_state = self
            .get_hard_state()
            .map_err(|e| self.build_tikv_error(e))?
            .expect("hard state should exist");

        meta.index = hard_state.get_commit();
        meta.term = if meta.index == self.snapshot_metadata.get_index() {
            self.snapshot_metadata.get_term()
        } else {
            let entry = self
                .get_entry(meta.index)
                .map_err(|e| self.build_tikv_error(e))?
                .unwrap_or_else(|| panic!("entry with {} should exists", meta.index));
            entry.term
        };
        let conf_state = self
            .get_conf_state()
            .map_err(|e| self.build_tikv_error(e))?
            .expect("conf state should exist");
        meta.set_conf_state(conf_state);
        if snapshot.get_metadata().index < request_index {
            snapshot.mut_metadata().index = request_index;
        }
        arcgraph_log::raft_debug!("snapshot={:?}", snapshot);
        // arcgraph_log::raft_debug!("snapshot bt={:?}", backtrace::Backtrace::new());
        Ok(snapshot)
    }
}

#[cfg(test)]
mod tests {

    use std::os::raw;

    use data_type::LogEntry;
    use protobuf::Message;
    use tikv_raft::{
        prelude::{ConfState, HardState, Snapshot, SnapshotMetadata},
        Storage,
    };

    use super::SledStorage;
    use crate::storage::LogStore;
    use prost::bytes::Bytes;
    use tikv_raft::prelude::Entry;

    fn new_store() -> SledStorage {
        let graph_id = 1;
        let partition_id = 2;
        let peer_id = 3;
        let folder = Some(String::from("/tmp/raft_log"));
        SledStorage::new(graph_id, partition_id, peer_id, folder)
    }

    fn build_entries(idx_list: Vec<u64>) -> Vec<Entry> {
        let mut entries = vec![];
        for idx in idx_list.iter() {
            entries.push(Entry {
                index: *idx,
                term: *idx,
                data: Bytes::from(bincode::serialize(&LogEntry::default()).unwrap()),
                ..Default::default()
            })
        }
        entries
    }

    #[test]
    fn test_store_new() {
        new_store();
    }

    #[test]
    fn test_init_state() {
        let store = new_store();
        if let Ok(rs) = store.initial_state() {
            println!(
                "hard_state={:?}, conf_state={:?}",
                rs.hard_state, rs.conf_state
            );
        }

        if let Ok(cs) = store.get_conf_state() {
            println!("conf_state={:?}", cs.unwrap());
        }

        if let Ok(hs) = store.get_hard_state() {
            println!("hard_state={:?}", hs.unwrap());
        }
    }

    #[test]
    fn test_set_hard_state() {
        let mut store = new_store();
        let old_hs = store.get_hard_state().unwrap().unwrap();
        println!("old hard state={:?}", old_hs);
        let hs = HardState {
            commit: 9,
            term: 10,
            vote: 200,
            ..Default::default()
        };
        if store.set_hard_state(&hs).is_ok() {
            let new_hs = store.get_hard_state().unwrap().unwrap();
            println!("new hard state={:?}", new_hs);
        }
    }

    #[test]
    fn test_set_conf_state() {
        let mut store = new_store();
        let old_cs = store.get_conf_state().unwrap().unwrap();
        println!("old conf state={:?}", old_cs);
        let cs = ConfState {
            voters: vec![1, 2, 3],
            learners: vec![4, 5, 6],
            ..Default::default()
        };
        if store.set_conf_state(&cs).is_ok() {
            let new_cs = store.get_conf_state().unwrap().unwrap();
            println!("new conf state={:?}", new_cs);
        }
    }

    #[test]
    fn test_append_entries() {
        let mut store = new_store();
        // test wrong first_index
        // let entry_idx_list = vec![0, 1, 2];
        // let entries = build_entries(entry_idx_list);
        // store.append(&entries);

        // test wrong last_index
        // let entry_idx_list = vec![u64::MAX - 3, u64::MAX -2, u64::MAX - 1];
        // let entries = build_entries(entry_idx_list);
        // store.append(&entries);

        // test append
        let entry_idx_list = vec![1, 2, 3, 4, 5];
        let entries = build_entries(entry_idx_list);
        store.append(&entries).unwrap();
        let first_index = store.get_first_index().unwrap();
        assert!(1 == first_index);
        let last_index = store.get_last_index().unwrap();
        println!("last_index={:?}", last_index);
        assert!(5 == last_index);
        let entries = store
            .get_entries(first_index, last_index + 1, None)
            .unwrap();
        println!("entries={:?}", entries);

        let entries = store
            .get_entries(first_index + 1, last_index + 1, 2)
            .unwrap();
        println!("entries={:?}", entries);

        // test append with override
        let entry_idx_list = vec![3, 4];
        let entries = build_entries(entry_idx_list);
        store.append(&entries).unwrap();
        let first_index = store.get_first_index().unwrap();
        assert!(1 == first_index);
        let last_index = store.get_last_index().unwrap();
        assert!(4 == last_index);
        let entries = store
            .get_entries(first_index, last_index + 1, None)
            .unwrap();
        println!("entries={:?}", entries);
    }

    #[test]
    fn test_create_snapshot() {
        let store = new_store();
        let snapshot = store.snapshot(0);
        println!("snapshot={:?}", snapshot);
    }

    #[test]
    fn test_apply_snapshot() {
        test_init_state();
        let mut store = new_store();
        let entry_idx_list = vec![1, 2, 3, 4, 5];
        let entries = build_entries(entry_idx_list);
        store.append(&entries).unwrap();

        // let mut snapshot = Snapshot::default();
        // test wrong first index
        // snapshot.set_metadata(SnapshotMetadata {
        //     index: 0,
        //     ..Default::default()
        // });
        // store.apply_snapshot(snapshot).unwrap();

        let mut snapshot = Snapshot::default();
        let mut meta = SnapshotMetadata {
            index: 1,
            term: 100,
            ..Default::default()
        };
        meta.set_conf_state(ConfState {
            voters: vec![1, 2, 3, 4],
            ..Default::default()
        });
        snapshot.set_metadata(meta);
        store.apply_snapshot(snapshot).unwrap();
        let first_index = store.get_first_index().unwrap();
        assert!(2 == first_index);
        let last_index = store.get_last_index().unwrap();
        let entries = store.get_entries(first_index, last_index + 1, None);
        println!("entries={:?}", entries);
        let hard_state = store.get_hard_state().unwrap();
        println!("hard_state={:?}", hard_state);
        let conf_state = store.get_conf_state().unwrap();
        println!("conf_state={:?}", conf_state);

        let mut snapshot = Snapshot::default();
        let mut meta = SnapshotMetadata {
            index: 5,
            term: 100,
            ..Default::default()
        };
        meta.set_conf_state(ConfState {
            voters: vec![1, 2, 3, 4],
            ..Default::default()
        });
        snapshot.set_metadata(meta);
        store.apply_snapshot(snapshot).unwrap();
        let first_index = store.get_first_index().unwrap();
        println!("first_index={}", first_index);
        let last_index = store.get_last_index().unwrap();
        println!("last_index={}", last_index);
        let entries = store.get_entries(first_index, last_index + 1, None);
        println!("entries={:?}", entries);
        let hard_state = store.get_hard_state().unwrap();
        println!("hard_state={:?}", hard_state);
        let conf_state = store.get_conf_state().unwrap();
        println!("conf_state={:?}", conf_state);
    }

    #[test]
    fn test_compact() {
        let mut store = new_store();
        let entry_idx_list = vec![1, 2, 3, 4, 5];
        let entries = build_entries(entry_idx_list);
        store.append(&entries).unwrap();
        store.compact(5).unwrap();
        let first_index = store.get_first_index().unwrap();
        assert!(5 == first_index);
        let last_index = store.get_last_index().unwrap();
        assert!(5 == last_index);
        let entries = store.get_entries(first_index, last_index, None).unwrap();
        println!("entries={:?}", entries);
    }

    #[test]
    fn test_get_term() {
        let mut store = new_store();
        let entry_idx_list = vec![1, 2, 3, 4, 5];
        let entries = build_entries(entry_idx_list);
        store.append(&entries).unwrap();
        let term = store.get_term(5).unwrap();
        assert!(5 == term);
    }

    #[test]
    fn test_first_last_index() {
        let graph_id = 1025;
        let partition_id = 0;
        let peer_ids = vec![
            7048549560733540344,
            7048549560733540345,
            7048549560733540346,
        ];
        for peer_id in peer_ids.into_iter() {
            let store = SledStorage::new(graph_id, partition_id, peer_id, None);
            let first_index = store.get_first_index().unwrap();
            let last_index = store.get_last_index().unwrap();
            let raw_entries = store
                .get_entries(first_index, last_index + 1, None)
                .unwrap();
            println!(
                "{} first_index={}, last_index={}",
                peer_id, first_index, last_index
            );
            println!("entries={:?}", raw_entries);

            let hs = store.get_hard_state().unwrap();
            let cs = store.get_conf_state().unwrap();
            println!("{} hs={:?}", peer_id, hs);
            println!("{} cs={:?}", peer_id, cs);
        }
    }
}
