use std::collections::HashMap;

use crate::parameter::Parameter;

// index representation is:
// i32 - min
// i32 - max
// u32 - number of entries
// multiple instances of:
// i32 - value
// u32 - count

struct BlockIndex {
    min: i32,
    max: i32,
    counts: HashMap<i32, u32>,
}

impl Into<Box<[u8]>> for BlockIndex {
    fn into(self) -> Box<[u8]> {
        let mut store: Vec<u8> = Vec::new();

        store.append(&mut self.min.to_le_bytes().to_vec());
        store.append(&mut self.max.to_le_bytes().to_vec());

        let mut num_entries = self.counts.len().to_le_bytes().to_vec();
        store.append(&mut num_entries);

        for (key, val) in &self.counts {
            store.append(&mut key.to_le_bytes().to_vec());
            store.append(&mut val.to_le_bytes().to_vec());
        }

        store.into_boxed_slice()
    }
}

impl From<&[u8]> for BlockIndex {
    fn from(value: &[u8]) -> Self {
        let min = i32::from_le_bytes(value[0..4].try_into().unwrap());
        let max = i32::from_le_bytes(value[4..8].try_into().unwrap());
        let num_entries = u32::from_le_bytes(value[8..12].try_into().unwrap());

        let mut map: HashMap<i32, u32> = HashMap::new();

        // starts reading at byte 12 offset
        // each entry is 8 bytes
        for i in 0..num_entries as usize {
            let entry_start = 12 + i * 8;
            let key_end = entry_start + 4;
            let value_start = key_end.clone();
            let entry_end = value_start + 4;
            let key = i32::from_le_bytes(value[entry_start..key_end].try_into().unwrap());
            let value = u32::from_le_bytes(value[value_start..entry_end].try_into().unwrap());

            map.insert(key, value);
        }

        BlockIndex {
            min: min,
            max: max,
            counts: map,
        }
    }
}

pub fn build_idx(_parameter: &Parameter, data: &[i32]) -> Box<[u8]> {
    // the index will have the min and max in the block along with counts for each of the elements
    // in the block.

    let mut min = i32::MAX;
    let mut max = i32::MIN;

    let mut map: HashMap<i32, u32> = HashMap::new();

    data.into_iter().for_each(|num| {
        if *num > max {
            max = *num;
        }

        if *num < min {
            min = *num;
        }

        match map.get(num) {
            Some(count) => map.insert(*num, count + 1),
            None => map.insert(*num, 1),
        };
    });

    BlockIndex {
        min: min,
        max: max,
        counts: map,
    }
    .into()
}

pub fn query_idx(_parameter: &Parameter, index: &[u8], query: &i32) -> Option<u64> {
    // check the index to see if the value is in range, otherwise, return None

    let block_idx: BlockIndex = index.into();

    if *query >= block_idx.min && *query <= block_idx.max {
        let result = match block_idx.counts.get(query) {
            Some(count) => Some(*count as u64),
            None => None,
        };

        return result;
    }

    Some(0)
}
