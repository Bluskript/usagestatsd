use std::{
    convert::TryInto,
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use sled::{self, Config};

pub struct Store {
    db: sled::Db,
    last_opened_tree: sled::Tree,
}

impl Store {
    pub fn new() -> Result<Store, Box<dyn std::error::Error>> {
        let db = Config::new()
            .flush_every_ms(Some(4000))
            .path("./usagestatsstore")
            .open()
            .unwrap();
        let last_opened_tree = db.open_tree("last_opened")?;
        Ok(Store {
            db,
            last_opened_tree,
        })
    }

    pub fn update_last_opened(
        &mut self,
        pkg: &std::string::String,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let current_time = Duration::as_secs(&SystemTime::now().duration_since(UNIX_EPOCH)?);
        self.last_opened_tree
            .insert(pkg.as_str(), &current_time.to_be_bytes())?;
        Ok(())
    }

    pub fn get_least_used(&mut self) -> Result<Vec<(String, u64)>, Box<dyn std::error::Error>> {
        let mut v: Vec<_> = self
            .last_opened_tree
            .iter()
            .flatten()
            .map(|res| {
                (
                    String::from_utf8(res.0.to_vec()).unwrap(),
                    u64::from_be_bytes(res.1.as_ref().try_into().unwrap()),
                )
            })
            .collect();
        v.sort_by_key(|p| p.1);
        v.reverse();
        Ok(v)
    }
}
