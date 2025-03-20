use std::collections::{HashMap, HashSet};
use dioxus::logger::tracing;
use dioxus::prelude::*;
use crate::CarTree;

pub fn mst_diff(
    target_tree: Signal<Option<CarTree>>,
    source_tree: Signal<Option<CarTree>>,
) -> () {
    let target_tree_ref = target_tree.as_ref().unwrap();
    let source_tree_ref = source_tree.as_ref().unwrap();
    let (target_roots, target_blocks, target_entries) = {
        (&target_tree_ref.roots, &target_tree_ref.blocks, &target_tree_ref.mst_entries)
    };
    let (source_roots, source_blocks, source_entries) = {
        (&source_tree_ref.roots, &source_tree_ref.blocks, &source_tree_ref.mst_entries)
    };
    let target_block_mapping = target_blocks
        .iter()
        .map(|block| (block.cid.to_owned(), block.cli_ipld_json.to_owned()))
        .collect::<HashMap<_, _>>();

    let source_block_mapping = source_blocks
        .iter()
        .map(|block| (block.cid.to_owned(), block.cli_ipld_json.to_owned()))
        .collect::<HashMap<_, _>>();

    let target_entry_cids = target_entries
        .iter()
        .map(|entry| &entry.record_cid)
        .collect::<HashSet<_>>();

    let source_entry_cids = source_entries
        .iter()
        .map(|entry| &entry.record_cid)
        .collect::<HashSet<_>>();

    let new_addition_cids = source_entry_cids
        .difference(&target_entry_cids)
        .collect::<HashSet<_>>();
    let new_removal_cids = target_entry_cids
        .difference(&source_entry_cids)
        .collect::<HashSet<_>>();

    tracing::debug!("new_addition_cids: {:?}", new_addition_cids);
    for cid in new_addition_cids {
        let new_block = source_block_mapping.get(*cid).unwrap_or(&None);
        tracing::debug!("\t{cid}: {new_block:?}");
    }
    tracing::debug!("new_removal_cids: {:?}", new_removal_cids);
    for cid in new_removal_cids {
        let deleted = target_block_mapping.get(*cid).unwrap_or(&None);
        tracing::debug!("\t{cid}: {deleted:?}");
    }
}

#[cfg(test)]
mod tests {
    use crate::load_car;
    use super::*;

    async fn get_older_tree() -> CarTree {
        let bytes = include_bytes!("../test_data/2025-03-18-repo.car").to_vec();
        load_car(bytes, Some(String::from("Older Tree"))).await.unwrap()
    }

    async fn get_newer_tree() -> CarTree {
        let bytes = include_bytes!("../test_data/2025-03-20-repo.car").to_vec();
        load_car(bytes, Some(String::from("Newer Tree"))).await.unwrap()
    }
}