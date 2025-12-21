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
async fn main() -> Result<()> {
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
