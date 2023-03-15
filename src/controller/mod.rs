use std::marker::PhantomData;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use anyhow::{Context, Result};

pub struct Db {
    path: PathBuf,
    column_families: Vec<String>,
    db: Arc<rocksdb::DB>,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self> {
        let options = rocksdb::Options::default();
        let column_families =
            rocksdb::DB::list_cf(&options, &path).context("Failed to read column families")?;

        let db = rocksdb::DB::open_cf(&options, &path, &column_families)
            .map(Arc::new)
            .context("Failed to open DB")?;

        Ok(Self {
            path,
            column_families,
            db,
        })
    }

    pub fn path(&self) -> &Path {
        self.path.as_ref()
    }

    pub fn column_families(&self) -> &[String] {
        &self.column_families
    }

    pub fn get_cf_handle(&self, cf_name: &str) -> Result<CfHandle<'_>> {
        let handle = self
            .db
            .cf_handle(cf_name)
            .context("Column family not found")?;

        let (key_repr, value_repr): (&dyn ValueRepr, &dyn ValueRepr) = match cf_name {
            "archives" => (&UintRepr::<u32>(PhantomData), &BlobRepr),
            "key_blocks" => (&UintRepr::<u32>(PhantomData), &BlockIdFullRepr),
            "shard_states" => (&BlockIdShortRepr, &ShardStateRepr),
            "prev1" | "prev2" | "next1" | "next2" => (&HexRepr, &BlockIdFullRepr),
            "package_entries" => (&PackageEntryIdRepr, &BlobRepr),
            "node_states" => (&NodeStatesRepr, &NodeStatesRepr),
            _ => (&HexRepr, &HexRepr),
        };

        Ok(CfHandle {
            handle,
            key_repr,
            value_repr,
        })
    }

    pub fn iter(&self, cf_handle: CfHandle<'_>) -> CfIterator<'_> {
        let iter = self
            .db
            .iterator_cf(&cf_handle.handle, rocksdb::IteratorMode::Start);
        CfIterator {
            iter,
            key_repr: cf_handle.key_repr,
            value_repr: cf_handle.value_repr,
        }
    }
}

pub struct CfHandle<'a> {
    handle: Arc<rocksdb::BoundColumnFamily<'a>>,
    key_repr: &'static dyn ValueRepr,
    value_repr: &'static dyn ValueRepr,
}

pub struct CfIterator<'a> {
    iter: rocksdb::DBIterator<'a>,
    key_repr: &'static dyn ValueRepr,
    value_repr: &'static dyn ValueRepr,
}

impl<'a> Iterator for CfIterator<'a> {
    type Item = (String, String);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next()? {
            Ok((key, value)) => {
                let key_repr = self.key_repr.repr_value(&key, &key);
                let value_repr = self.value_repr.repr_value(&key, &value);
                Some((key_repr, value_repr))
            }
            Err(_) => None,
        }
    }
}

trait ValueRepr {
    fn repr_value(&self, ctx: &[u8], value: &[u8]) -> String;
}

struct UintRepr<T>(PhantomData<T>);

impl ValueRepr for UintRepr<u32> {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        if value.len() != 4 {
            return format!("<invalid {}>", hex::encode(value));
        }
        u32::from_be_bytes(value.try_into().unwrap()).to_string()
    }
}

struct HexRepr;

impl ValueRepr for HexRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        hex::encode(value)
    }
}

struct BlockIdShortRepr;

impl ValueRepr for BlockIdShortRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        if value.len() != 16 {
            return format!("<invalid {}>", hex::encode(value));
        }

        let workchain = i32::from_be_bytes(value[0..4].try_into().unwrap());
        let shard = u64::from_be_bytes(value[4..12].try_into().unwrap());
        let seqno = u32::from_be_bytes(value[12..16].try_into().unwrap());
        format!("{workchain:2}:{shard:016x}:{seqno}")
    }
}

struct BlockIdFullRepr;

impl ValueRepr for BlockIdFullRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        if value.len() != 80 {
            return format!("<invalid {}>", hex::encode(value));
        }

        let workchain = i32::from_be_bytes(value[0..4].try_into().unwrap());
        let shard = u64::from_be_bytes(value[4..12].try_into().unwrap());
        let seqno = u32::from_be_bytes(value[12..16].try_into().unwrap());
        let root_hash = hex::encode(&value[16..48]);
        let file_hash = hex::encode(&value[48..80]);
        format!("{workchain:2}:{shard:016x}:{seqno}:{root_hash}:{file_hash}")
    }
}

struct ShardStateRepr;

impl ValueRepr for ShardStateRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        if value.len() != 32 * 3 {
            return format!("<invalid {}>", hex::encode(value));
        }

        let state_root = hex::encode(&value[0..32]);
        let root_hash = hex::encode(&value[32..64]);
        let file_hash = hex::encode(&value[64..96]);
        format!("state_root: {state_root}, root_hash: {root_hash}, file_hash: {file_hash}")
    }
}

struct BlobRepr;

impl ValueRepr for BlobRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        format!("<{} bytes>", value.len())
    }
}

struct PackageEntryIdRepr;

impl ValueRepr for PackageEntryIdRepr {
    fn repr_value(&self, _: &[u8], value: &[u8]) -> String {
        if value.len() != 16 + 32 + 1 {
            return format!("<invalid {}>", hex::encode(value));
        }

        let workchain = i32::from_be_bytes(value[0..4].try_into().unwrap());
        let shard = u64::from_be_bytes(value[4..12].try_into().unwrap());
        let seqno = u32::from_be_bytes(value[12..16].try_into().unwrap());
        let root_hash = hex::encode(&value[16..48]);
        let package_type = match value[48] {
            0 => "block",
            1 => "proof",
            2 => "proof_link",
            _ => "unknown",
        };
        format!("{workchain:2}:{shard:016x}:{seqno}:{root_hash}: {package_type}")
    }
}

struct NodeStatesRepr;

impl ValueRepr for NodeStatesRepr {
    fn repr_value(&self, ctx: &[u8], value: &[u8]) -> String {
        const HISTORICAL_SYNC_LOW: &[u8] = b"background_sync_low";
        const HISTORICAL_SYNC_HIGH: &[u8] = b"background_sync_high";

        const LAST_UPLOADED_ARCHIVE: &[u8] = b"last_uploaded_archive";

        const LAST_MC_BLOCK_ID: &[u8] = b"LastMcBlockId";
        const INIT_MC_BLOCK_ID: &[u8] = b"InitMcBlockId";
        const SHARDS_CLIENT_MC_BLOCK_ID: &[u8] = b"ShardsClientMcBlockId";

        const DB_VERSION_KEY: &[u8] = b"db_version";

        if ctx == value {
            String::from_utf8_lossy(value).to_string()
        } else {
            match ctx {
                HISTORICAL_SYNC_LOW
                | HISTORICAL_SYNC_HIGH
                | LAST_MC_BLOCK_ID
                | INIT_MC_BLOCK_ID
                | SHARDS_CLIENT_MC_BLOCK_ID => BlockIdFullRepr.repr_value(ctx, value),
                LAST_UPLOADED_ARCHIVE => UintRepr::<u32>(PhantomData).repr_value(ctx, value),
                DB_VERSION_KEY if value.len() != 3 => {
                    format!("<invalid version {}>", hex::encode(value))
                }
                DB_VERSION_KEY => {
                    format!("{}.{}.{}", value[0], value[1], value[2])
                }
                _ => hex::encode(value),
            }
        }
    }
}
