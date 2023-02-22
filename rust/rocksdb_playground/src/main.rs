#![feature(test)]

use std::time::Duration;
use std::time::Instant;

use rocksdb::Cache;
use rocksdb::{
    ColumnFamily, ColumnFamilyDescriptor, DBWithThreadMode, Options, SingleThreaded, DB, BlockBasedOptions, WriteBatch,
};

use rand::prelude::*;
use rand::thread_rng;
use rand::seq::SliceRandom;
use sha2::Digest;

extern crate test;

const PATH: &str = "/Users/pugachag/Data/rocksdb/playground";
const COL: &str = "col1";
const KEY_SIZE: usize = 300;
const BLOCK_SIZE: usize = 10_000;
const BLOCK_CNT: usize = 500;

type CryptoHash = [u8; 32];

fn calc_hash(data: &[u8]) -> CryptoHash {
    sha2::Sha256::digest(data).into()
}

fn block_hash(block_id: usize) -> CryptoHash {
    let s = format!("block{block_id}");
    calc_hash(s.as_bytes())
}

fn key_bytes(block_id: usize, key_id: usize) -> Vec<u8> {
    let mut ret = vec![];
    ret.extend_from_slice(&block_hash(block_id));
    for i in key_id.. {
        ret.extend_from_slice(i.to_string().as_bytes());
        if ret.len() >= KEY_SIZE {
            break;
        }
    }
    ret
}

fn value_bytes(key_id: usize) -> Vec<u8> {
    let mut ret = vec![];
    ret.extend_from_slice(&calc_hash(format!("value{key_id}").as_bytes()));
    ret
}

struct Env {
    db: DBWithThreadMode<SingleThreaded>,
}

impl Env {
    fn new() -> Self {
        Env { db: Self::neard_db() }
    }

    fn custom_db() -> DBWithThreadMode<SingleThreaded> {
        let mut cf_opts = Options::default();
        let mut factory = BlockBasedOptions::default();
        //factory.set_block_size(16000);
        factory.disable_cache();
        //factory.set_bloom_filter(bits_per_key, block_based);
        cf_opts.set_block_based_table_factory(&factory);
        let cf = ColumnFamilyDescriptor::new(COL, cf_opts);

        let mut db_opts = Options::default();
        db_opts.create_missing_column_families(true);
        db_opts.create_if_missing(true);

        let db = DB::open_cf_descriptors(&db_opts, PATH, vec![cf]).unwrap();
        db
    }

    fn neard_db() -> DBWithThreadMode<SingleThreaded> {
        let mut block_opts = BlockBasedOptions::default();
        block_opts.set_block_size(16000);
        block_opts
            .set_block_cache(&Cache::new_lru_cache(512_000_000).unwrap());
        block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
        block_opts.set_cache_index_and_filter_blocks(true);
        //block_opts.set_bloom_filter(16.0, false);
        block_opts.set_bloom_filter(10.0, true);

        let mut cf_opts = Options::default();
        // fn set_compression_options(opts: &mut Options) {
        cf_opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        cf_opts.set_bottommost_compression_type(rocksdb::DBCompressionType::Zstd);
        // RocksDB documenation says that 16KB is a typical dictionary size.
        // We've empirically tuned the dicionary size to twice of that 'typical' size.
        // Having train data size x100 from dictionary size is a recommendation from RocksDB.
        // See: https://rocksdb.org/blog/2021/05/31/dictionary-compression.html?utm_source=dbplatz
        let dict_size = 2 * 16384;
        let max_train_bytes = dict_size * 100;
        // We use default parameters of RocksDB here:
        //      window_bits is -14 and is unused (Zlib-specific parameter),
        //      compression_level is 32767 meaning the default compression level for ZSTD,
        //      compression_strategy is 0 and is unused (Zlib-specific parameter).
        // See: https://github.com/facebook/rocksdb/blob/main/include/rocksdb/advanced_options.h#L176:
        cf_opts.set_bottommost_compression_options(
            /*window_bits */ -14, /*compression_level */ 32767,
            /*compression_strategy */ 0, dict_size, /*enabled */ true,
        );
        cf_opts.set_bottommost_zstd_max_train_bytes(max_train_bytes, true);

        cf_opts.set_level_compaction_dynamic_level_bytes(true);
        cf_opts.set_block_based_table_factory(&block_opts);
        cf_opts.optimize_level_style_compaction(128_000_000);
        cf_opts.set_target_file_size_base(64_000_000);

        let cf = ColumnFamilyDescriptor::new(COL, cf_opts);

        let mut opts = Options::default();

        //set_compression_options(&mut opts);
        opts.create_missing_column_families(true);
        opts.create_if_missing(true);
        opts.set_use_fsync(false);
        opts.set_max_open_files(10_000);
        opts.set_keep_log_file_num(1);
        opts.set_bytes_per_sync(1000_000);
        opts.set_write_buffer_size(256_000_000);
        opts.set_max_bytes_for_level_base(256_000_000);

        opts.increase_parallelism(2);
        opts.set_max_total_wal_size(1_000_000_000);

        DB::open_cf_descriptors(&opts, PATH, vec![cf]).unwrap()
    }

    fn cf(&self) -> &ColumnFamily {
        self.db.cf_handle(COL).unwrap()
    }

    fn destroy(self) {
        println!("destroy");
        drop(self.db);
        DB::destroy(&Options::default(), PATH).unwrap();
    }

    fn rewrite_all() {
        Self::new().destroy();
        let env = Self::new();
        env.write_all_blocks();
        env.compact();
    }

    fn write_all_blocks(&self) {
        for block_id in 0..BLOCK_CNT {
            self.write_block(block_id);
        }
    }

    fn write_block(&self, block_id: usize) {
        println!("write_block {block_id}");
        let mut batch = WriteBatch::default();
        for key_id in Self::block_keys(block_id, true) {
            batch.put_cf(self.cf(), key_bytes(block_id, key_id), value_bytes(key_id));
        }
        self.db.write(batch).unwrap();
    }

    fn read(&self, key: &[u8]) -> bool {
        self.db
            .get_cf(self.cf(), key)
            .unwrap().is_some()
    }
    
    fn compact(&self) {
        println!("compact");
        let none = Option::<&[u8]>::None;
        self.db.compact_range_cf(self.cf(), none, none);
    }

    fn block_keys(_block_id: usize, present: bool) -> Vec<usize> {
        (0..2*BLOCK_SIZE).skip(if present {0} else {1}).step_by(2).collect()
    }

    fn print_measurements(mut measurements: Vec<Duration>) {
        let avg = measurements.iter().sum::<Duration>() / measurements.len() as u32; 
        println!("avg = {avg:?}");
        measurements.sort();
        for p in [10, 50, 90, 95, 99, 100] {
            let i = std::cmp::min(p * measurements.len() / 100, measurements.len() - 1);
            println!("p{p} = {:?}", measurements[i]);
        }
    }

    fn measure_read(&self, present: bool) {
        const READ_CNT: usize = 100_000;
        let mut keys = (0..BLOCK_CNT).flat_map(|block_id|
            Self::block_keys(block_id, present).into_iter().map(move |key_id| (block_id, key_id))
        ).collect::<Vec<_>>();
        println!("Total size: {}MB", (BLOCK_CNT * BLOCK_SIZE * (KEY_SIZE + 32)) / 1000_000);
        println!("Reading {} out of {} = {}%", READ_CNT, keys.len(), 100 * READ_CNT / keys.len());
        assert!(keys.len() >= READ_CNT);
        keys.shuffle(&mut StdRng::seed_from_u64(42));
        let mut measurements = vec![];
        for (block_id, key_id) in keys.into_iter().take(READ_CNT) {
            let key = key_bytes(block_id, key_id);
            let start = Instant::now();
            self.read(&key);
            measurements.push(start.elapsed());
        }
        Self::print_measurements(measurements);
    }
}

fn main() {
    //Env::rewrite_all();
    let env = Env::new();
    env.measure_read(false);
}

