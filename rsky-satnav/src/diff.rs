use std::collections::HashMap;
use dioxus::logger::tracing;
use dioxus::prelude::*;
use ipld_core::cid::Cid;
use crate::{read_commit, walk_mst_entries, BlockMap, CarTree, MstEntry};


#[derive(Debug)]
enum RecordDiffType {
    Identical(String),
    Added(String),
    Updated(String, String),
    Removed(String)
}

#[derive(Debug)]
struct RecordDiff {
    collection: String,
    record_key: String,
    diff_type: RecordDiffType,
    data: String,
}

pub fn mst_diff(
    target_tree: Signal<Option<CarTree>>,
    source_tree: Signal<Option<CarTree>>,
) -> Vec<MstEntry> {
    let target_entries = &target_tree.as_ref().unwrap().mst_entries;
    let source_entries = &source_tree.as_ref().unwrap().mst_entries;

    let make_key_fn = |collection: &String, record_key: &String| {
        format!("{collection}/{record_key}")
    };

    let mut source_entries = source_entries
        .into_iter()
        .map(|entry| {
            (make_key_fn(&entry.collection, &entry.rkey), (entry.record_cid.to_owned(), entry.record_cli_json.to_owned()))
        })
        .collect::<HashMap<_, _>>();

    // ZJ-TODO: we're assuming that the values returned by walk_mst_entries are sorted alphabetically
    //          this should be enforced by tests and/or assertions but we're timeboxed

    let mut diffs = vec![];
    for entry in target_entries {
        let target_collection = &entry.collection;
        let target_record_key = &entry.rkey;
        let target_record_cid = &entry.record_cid;

        let target_key = make_key_fn(&target_collection, &target_record_key);

        // If both trees contain the record, it's either unchanged or updated
        if let Some((source_record_cid, data)) = source_entries.remove(&target_key) {
            let identical = source_record_cid == *target_record_cid;
            diffs.push(RecordDiff {
                collection: target_collection.to_owned(),
                record_key: target_record_key.to_owned(),
                diff_type: if identical {
                    RecordDiffType::Identical(target_record_cid.to_owned())
                } else {
                    RecordDiffType::Updated(target_record_cid.to_owned(), source_record_cid)
                },
                data,
            });
        }
        // If the source tree did not contain the record, it's been deleted
        else {
            diffs.push(RecordDiff {
                collection: target_collection.to_owned(),
                record_key: target_record_key.to_owned(),
                diff_type: RecordDiffType::Removed(target_record_cid.to_owned()),
                data: String::new(),
            });
        }
    }

    // If we have any remaining source tree entries, those are new
    for (record_key, (cid, data)) in source_entries {
        let key_split = record_key.split("/").collect::<Vec<_>>();
        let collection = key_split[0];
        let record_key = key_split[1];
        diffs.push(RecordDiff {
            collection: collection.to_string(),
            record_key: record_key.to_string(),
            diff_type: RecordDiffType::Added(cid),
            data,
        });
    }

    construct_mst_from_diffs(diffs)
}

pub fn construct_mst_from_diffs(diffs: Vec<RecordDiff>) -> Vec<MstEntry> {
    let mut mst_entries = vec![];

    for diff in diffs {
        match diff.diff_type {
            RecordDiffType::Identical(_) | RecordDiffType::Removed(_) => {},
            RecordDiffType::Added(cid) | RecordDiffType::Updated(_, cid) => {
                mst_entries.push(MstEntry {
                    collection: diff.collection.to_owned(),
                    rkey: diff.record_key.to_owned(),
                    record_cid: cid.to_owned(),
                    record_cli_json: diff.data.to_owned(),
                });
            },
        }
    }

    mst_entries
}