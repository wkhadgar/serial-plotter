use super::{map_io_error, map_serde_error};
use crate::core::error::AppResult;
use serde::Serialize;
use std::fs;
use std::path::Path;

pub(super) fn create_dir_all(path: &Path, context: &str) -> AppResult<()> {
    fs::create_dir_all(path).map_err(|error| map_io_error(context, &error))
}

pub(super) fn write_string(path: &Path, contents: &str, context: &str) -> AppResult<()> {
    fs::write(path, contents).map_err(|error| map_io_error(context, &error))
}

pub(super) fn write_json_pretty<T: Serialize>(
    path: &Path,
    value: &T,
    serialize_context: &str,
    write_context: &str,
) -> AppResult<()> {
    let payload = serde_json::to_string_pretty(value)
        .map_err(|error| map_serde_error(serialize_context, &error))?;
    write_string(path, &payload, write_context)
}

pub(super) fn read_to_string(path: &Path, context: &str) -> AppResult<String> {
    fs::read_to_string(path).map_err(|error| map_io_error(context, &error))
}

pub(super) fn remove_dir_if_exists(path: &Path, context: &str) -> AppResult<()> {
    if !path.exists() {
        return Ok(());
    }

    fs::remove_dir_all(path).map_err(|error| map_io_error(context, &error))
}
