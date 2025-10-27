use skip_list::user::{build_idx, query_idx};
use std::fs::File;
use std::io;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use skip_list::parameter::Parameter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        eprintln!("Usage: <program> <file> <factor-size> <factor-skip>");
        std::process::exit(1);
    }

    let parameter = Parameter {
        factor_size: args[2].parse().unwrap_or(1),
        factor_skip: args[3].parse().unwrap_or(1)
    };
    let file_stem = &args[1];

    let data_path = PathBuf::from(format!("./data/{}.data", file_stem));
    let query_path = PathBuf::from(format!("./data/{}.query", file_stem));

    let data = load_data(&data_path)?;
    let mut indexes: Vec<Box<[u8]>> = Vec::with_capacity(data.len());
    let mut total_index_size_bytes = 0usize;

    for block in data.iter() {
        let idx = build_idx(&parameter, block);
        total_index_size_bytes += idx.len();
        indexes.push(idx);
    }

    let queries = load_query(&query_path)?;
    let mut n_skipped = 0u64;

    for query in queries.as_slice() {
        let mut expected_result = 0u64;
        let mut user_result = 0u64;

        for block_i in 0..data.len() {
            let mut block_result = 0u64;
            for num in data[block_i].as_slice() {
                if query == num {
                    block_result +=  1;
                }
            }
            expected_result += block_result;

            match query_idx(&parameter, &indexes[block_i], query) {
                None => {
                    user_result += block_result;
                }
                Some(idx_hint) => {
                    user_result += idx_hint;
                    n_skipped += 1;
                }
            }
        }
        assert_eq!(expected_result, user_result, "Wrong answer!");
    }
    let total_index_size_kb = total_index_size_bytes / 1024;
    let max_score = parameter.factor_skip * queries.len() as i64 * data.len() as i64;
    let score = (n_skipped as i64 * parameter.factor_skip) - (total_index_size_kb as i64 * parameter.factor_size);
    println!("storage size: {} {}", total_index_size_kb, total_index_size_kb as i64 * parameter.factor_size);
    println!("num skips: {} {}", n_skipped, n_skipped as i64 * parameter.factor_skip);
    println!("total score: {}", score as f64 / max_score as f64 * 100f64);
    Ok(())
}

fn load_data(file_path: &Path) -> io::Result<Vec<Vec<i32>>> {
    let mut file = BufReader::new(File::open(file_path)?);

    let mut buf8 = [0u8; 8];
    file.read_exact(&mut buf8)?;
    let n_block = u64::from_le_bytes(buf8) as usize;
    file.read_exact(&mut buf8)?;
    let chunk_size = u64::from_le_bytes(buf8) as usize;
    let n = n_block.checked_mul(chunk_size).expect("n_block * chunk_size overflow");
    let mut blocks: Vec<Vec<i32>> = Vec::with_capacity(n_block);

    let mut buf4 = [0u8; 4];
    for block_idx in 0..n_block {
        let mut chunk = Vec::with_capacity(chunk_size);
        for _ in 0..chunk_size {
            // stop exactly at n elements
            if chunk.len() + block_idx * chunk_size >= n {
                break;
            }
            file.read_exact(&mut buf4)?;
            chunk.push(i32::from_le_bytes(buf4));
        }
        blocks.push(chunk);
    }

    Ok(blocks)
}

fn load_query(file_path: &Path) -> io::Result<Vec<i32>> {
    let mut file = BufReader::new(File::open(file_path)?);

    let mut buf8 = [0u8; 8];
    file.read_exact(&mut buf8)?;
    let n = u64::from_le_bytes(buf8) as usize;

    let mut data = Vec::with_capacity(n);
    let mut buf4 = [0u8; 4];
    for _ in 0..n {
        file.read_exact(&mut buf4)?;
        data.push(i32::from_le_bytes(buf4));
    }

    Ok(data)
}