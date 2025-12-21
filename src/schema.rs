use crate::planner::RenameOp;
use anyhow::Result;
use chrono::{DateTime, Local};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

pub fn apply_schema(files: Vec<PathBuf>, schema: &str) -> Result<Vec<RenameOp>> {
    let mut plan = Vec::new();
    let mut index = 0;

    for file in files {
        // Skip renaming if it's a directory? The walker includes files only currently.
        let new_name = process_schema(&file, schema, index)?;
        let new_path = file.parent().unwrap().join(new_name);

        plan.push(RenameOp {
            original: file,
            new: new_path,
            reason: format!("Schema: {}", schema),
        });
        index += 1;
    }
    Ok(plan)
}

fn process_schema(path: &PathBuf, schema: &str, index: usize) -> Result<String> {
    let mut result = schema.to_string();

    // {orig}
    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    result = result.replace("{orig}", &stem);

    // {ext}
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    result = result.replace("{ext}", &ext);

    // {A-*}
    if result.contains("{A-*}") {
        let suffix = to_alphabetic(index);
        result = result.replace("{A-*}", &suffix);
    }

    // {1-*}
    if result.contains("{1-*}") {
        result = result.replace("{1-*}", &(index + 1).to_string());
    }

    // {date}
    if result.contains("{date}") {
        let metadata = path.metadata()?;
        let modified: DateTime<Local> = metadata.modified()?.into();
        result = result.replace("{date}", &modified.format("%Y-%m-%d").to_string());
    }

    // {hash}
    if result.contains("{hash}") {
        let hash = compute_short_hash(path)?;
        result = result.replace("{hash}", &hash);
    }

    if !ext.is_empty() && !schema.contains("{ext}") {
        result = format!("{}.{}", result, ext);
    }

    Ok(result)
}

fn to_alphabetic(n: usize) -> String {
    let mut m = n + 1;
    let mut result = String::new();
    while m > 0 {
        m -= 1;
        let remainder = m % 26;
        let char_code = (b'A' + remainder as u8) as char;
        result.insert(0, char_code);
        m /= 26;
    }
    result
}

fn compute_short_hash(path: &PathBuf) -> Result<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 1024];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
    }
    let result = hasher.finalize();
    let hex = hex::encode(result);
    Ok(hex[0..8].to_string())
}
