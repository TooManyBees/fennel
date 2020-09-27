use std::io::{Error, ErrorKind, Result};
use std::fs::File;

use crate::PlayerRecord;

pub fn save(name: &str, player_record: PlayerRecord) -> Result<()> {
    let mut f = File::create(PlayerRecord::file_path(name))?;
    serde_json::to_writer_pretty(&mut f, &player_record)
        .map_err(|e| Error::new(ErrorKind::Other, e))?;
    f.sync_data()
}
