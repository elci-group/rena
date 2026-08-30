mod curly_expand;
use anyhow::Result;
use clap::Parser;
use std::path::PathBuf;

mod llm;
mod planner;
mod schema;

#[derive(Parser, Debug)]
#[command(name = "rena")]
#[command(about = "Recursive, expressive rename utility", long_about = None)]
pub struct Cli {
    /// Target directory or file
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Recursive renaming
    #[arg(long, short)]
    pub recursive: bool,

    /// Expressive mode (content-based naming)
    #[arg(long, short)]
    pub expressive: bool,

    /// Contextual mode (code-aware naming)
    #[arg(long, short = 'c')]
    pub contextual: bool,

    /// Apply the changes (default is dry-run)
    #[arg(long)]
    pub apply: bool,

    /// Schema pattern for recursive renaming
    #[arg(long, short = 's')]
    pub schema: Option<String>,

    /// Listen mode for STT (placeholder for now)
    #[arg(long)]
    pub listen: bool,
}

#[tokio::main]
async fn __curly_original_main() -> Result<()> {
    let args = Cli::parse();

    if args.listen {
        println!("Listening mode not yet implemented.");
        return Ok(());
    }

    let plan = planner::generate_plan(&args).await?;

    if plan.is_empty() {
        println!("No files to rename.");
        return Ok(());
    }

    planner::present_plan(&plan);

    if args.apply {
        planner::execute_plan(&plan)?;
        println!("Renaming complete.");
    } else {
        println!("\nDry-run complete. Use --apply to execute.");
    }

    Ok(())
}

fn main() {
    let raw_args: Vec<String> = std::env::args().collect();
    let mut positions: Vec<usize> = Vec::new();
    let mut fields: Vec<Vec<String>> = Vec::new();
    for (__i, __a) in raw_args.iter().enumerate() {
        if __a == "--schema" {
            if let Some(__v) = raw_args.get(__i + 1) {
                positions.push(__i + 1);
                fields.push(curly_expand::expand_or_literal(__v));
            }
            break;
        } else if let Some(__v) = __a.strip_prefix("--schema=") {
            positions.push(__i);
            fields.push(
                curly_expand::expand_or_literal(__v)
                    .into_iter()
                    .map(|v| format!("--schema={}", v))
                    .collect(),
            );
            break;
        }
    }

    if fields.is_empty() || fields.iter().all(|f| f.len() <= 1) {
        __curly_original_main();
        return;
    }

    let combos = curly_expand::cartesian(&fields);
    let exe = std::env::current_exe().expect("resolve current exe");
    let mut had_failure = false;
    for combo in &combos {
        let mut new_args = raw_args.clone();
        for (slot, value) in positions.iter().zip(combo.iter()) {
            new_args[*slot] = value.clone();
        }
        let status = std::process::Command::new(&exe)
            .args(&new_args[1..])
            .status()
            .expect("failed to re-exec self");
        if !status.success() {
            had_failure = true;
        }
    }
    if had_failure {
        std::process::exit(1);
    }
}
