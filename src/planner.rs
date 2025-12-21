use crate::Cli;
use anyhow::Result;
use std::path::PathBuf;
use walkdir::WalkDir;

pub struct RenameOp {
    pub original: PathBuf,
    pub new: PathBuf,
    pub reason: String,
}

pub async fn generate_plan(args: &Cli) -> Result<Vec<RenameOp>> {
    // 1. Collect files
    let mut files = Vec::new();
    if args.path.is_file() {
        files.push(args.path.clone());
    } else {
        let walker = WalkDir::new(&args.path).sort_by_file_name();
        for entry in walker {
            match entry {
                Ok(e) => {
                    if e.file_type().is_file() {
                        files.push(e.path().to_path_buf());
                    }
                }
                Err(e) => eprintln!("Error reading entry: {}", e),
            }
        }
    }

    // Sort is handled by sort_by_file_name in WalkDir roughly, but for strict determinism we might want to sort by full path
    files.sort();

    let plan;

    if args.expressive || args.contextual {
        plan = crate::llm::propose_renames(files, args).await?;
    } else if let Some(schema) = &args.schema {
        plan = crate::schema::apply_schema(files, schema)?;
    } else if args.recursive {
        // If recursive is on but no schema provided, we can't really "rename" without a pattern.
        // Unless we interpret a positional arg as schema.
        // For now, let's just use "{orig}" (no-op) effectively, or error.
        // Let's assume the user might want to just list or doing something else.
        // But the prompt example `rena --recursive ~/Downloads ~/Downloads/file{A-*}`
        // implies the second arg is the schema.
        // Since I defined schema as a flag, I'll expect the flag.
        // If not, I'll use a placeholder or error.
        eprintln!("Warning: --recursive used without --schema. No renaming will occur.");
        plan = Vec::new();
    } else {
        plan = Vec::new();
    }

    Ok(plan)
}

pub fn present_plan(plan: &[RenameOp]) {
    println!("{:<50} -> {:<50} | {}", "Original", "New", "Reason");
    println!("{}", "-".repeat(120));
    for op in plan {
        let orig_name = op.original.display().to_string();
        let new_name = op.new.display().to_string();
        // Truncate if too long for display
        let orig_display = if orig_name.len() > 48 {
            format!("...{}", &orig_name[orig_name.len() - 45..])
        } else {
            orig_name
        };
        let new_display = if new_name.len() > 48 {
            format!("...{}", &new_name[new_name.len() - 45..])
        } else {
            new_name
        };

        println!(
            "{:<50} -> {:<50} | {}",
            orig_display, new_display, op.reason
        );
    }
}

pub fn execute_plan(plan: &[RenameOp]) -> Result<()> {
    for op in plan {
        if op.original != op.new {
            if let Some(parent) = op.new.parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Check for collision
            if op.new.exists() {
                // Simple safety: don't overwrite
                eprintln!("Skipping {}: Target exists", op.original.display());
                continue;
            }
            std::fs::rename(&op.original, &op.new)?;
        }
    }
    Ok(())
}
