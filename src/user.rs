use std::collections::HashMap;

use crate::parameter::Parameter;

// index representation is:
// multiple instances of:
// i32 - value
// u32 - count

struct BlockIndex {
    counts: HashMap<i32, u32>,
}

impl Into<Box<[u8]>> for BlockIndex {
    fn into(self) -> Box<[u8]> {
        let mut store: Vec<u8> = Vec::new();

        for (key, val) in &self.counts {
            store.append(&mut key.to_le_bytes().to_vec());
            store.append(&mut val.to_le_bytes().to_vec());
        }

        store.into_boxed_slice()
    }
}

impl From<&[u8]> for BlockIndex {
    fn from(value: &[u8]) -> Self {
        let mut map: HashMap<i32, u32> = HashMap::new();

        // starts reading at byte 0 offset
        // each entry is 8 bytes
        let mut i = 0;
        loop {
            let entry_start = i * 8;
            if entry_start >= value.len() {
                break;
            }
            let key_end = entry_start + 4;
            let value_start = key_end.clone();
            let entry_end = value_start + 4;
            let key = i32::from_le_bytes(value[entry_start..key_end].try_into().unwrap());
            let value = u32::from_le_bytes(value[value_start..entry_end].try_into().unwrap());

            map.insert(key, value);
            i += 1;
        }

        BlockIndex { counts: map }
    }
}

pub fn build_idx(_parameter: &Parameter, data: &[i32]) -> Box<[u8]> {
    // the index will have the min and max in the block along with counts for each of the elements
    // in the block.

    let mut map: HashMap<i32, u32> = HashMap::new();

    data.into_iter().for_each(|num| {
        match map.get(num) {
            Some(count) => map.insert(*num, count + 1),
            None => map.insert(*num, 1),
        };
    });

    BlockIndex { counts: map }.into()
}

pub fn query_idx(_parameter: &Parameter, index: &[u8], query: &i32) -> Option<u64> {
    // check the index to see if the value is in range, otherwise, return None

    let block_idx: BlockIndex = index.into();

    match block_idx.counts.get(query) {
        Some(count) => Some(*count as u64),
        None => Some(0),
    }
}
