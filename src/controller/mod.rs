use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

pub struct Db {
    path: PathBuf,
    column_families: Vec<String>,
    db: rocksdb::DB,
}

impl Db {
    pub fn open(path: PathBuf) -> Result<Self> {
        let options = rocksdb::Options::default();
        let column_families =
            rocksdb::DB::list_cf(&options, &path).context("Failed to read column families")?;

        let db =
            rocksdb::DB::open_cf(&options, &path, &column_families).context("Failed to open DB")?;

        Ok(Self {
            path,
            column_families,
            db,
        })
    }

    pub fn column_families(&self) -> &[String] {
        &self.column_families
    }
}
