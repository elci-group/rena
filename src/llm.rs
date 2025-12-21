use crate::planner::RenameOp;
use crate::Cli;
use anyhow::{Context, Result};
use reqwest::Client;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::path::PathBuf;

const MODEL_NAME: &str = "gemini-1.5-flash"; // Using 1.5 Flash as stable target

pub async fn propose_renames(files: Vec<PathBuf>, args: &Cli) -> Result<Vec<RenameOp>> {
    let api_key =
        env::var("GOOGLE_API_KEY").context("GOOGLE_API_KEY must be set for LLM features")?;
    let client = Client::new();

    if args.contextual {
        return contextual_rename(&client, &api_key, files).await;
    }

    if args.expressive {
        return expressive_rename(&client, &api_key, files).await;
    }

    Ok(vec![])
}

async fn contextual_rename(
    client: &Client,
    api_key: &str,
    files: Vec<PathBuf>,
) -> Result<Vec<RenameOp>> {
    // Prepare the list of files relative to the root (or just filenames if flat)
    // We assume files are paths. Let's convert to strings.
    let file_list: Vec<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let prompt = format!(
        "You are a code maintenance assistant. I will provide a list of filenames. \
        Rename them to be consistent, descriptive, and follow best practices for the codebase (snake_case, kebab-case, etc. as inferred). \
        Maintain file extensions. \
        Return a JSON object where keys are original filenames and values are new filenames. \
        Do not explain. \
        \n\nFiles:\n{}",
        file_list.join("\n")
    );

    let response = call_gemini(client, api_key, &prompt).await?;
    parse_response(files, response, "Contextual LLM")
}

async fn expressive_rename(
    client: &Client,
    api_key: &str,
    files: Vec<PathBuf>,
) -> Result<Vec<RenameOp>> {
    let mut ops = Vec::new();

    // Batch processing could be better, but for simplicity let's do a few at a time or loop.
    // To avoid hitting rate limits or context limits, let's process in chunks of 5.
    for chunk in files.chunks(5) {
        let mut prompt_parts = Vec::new();
        prompt_parts.push("Rename the following files based on their content. Use descriptive names. Keep extensions. Return JSON mapping {old: new}.".to_string());

        for file in chunk {
            let content = fs::read_to_string(file)
                .unwrap_or_else(|_| "Binary or unreadable content".to_string());
            let snippet = if content.len() > 1000 {
                &content[..1000]
            } else {
                &content
            };
            prompt_parts.push(format!(
                "File: {}\nContent:\n{}\n---",
                file.to_string_lossy(),
                snippet
            ));
        }

        let prompt = prompt_parts.join("\n");
        let response = call_gemini(client, api_key, &prompt).await?;
        let chunk_ops = parse_response(chunk.to_vec(), response, "Expressive LLM")?;
        ops.extend(chunk_ops);
    }

    Ok(ops)
}

async fn call_gemini(client: &Client, api_key: &str, prompt: &str) -> Result<String> {
    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
        MODEL_NAME, api_key
    );

    let body = json!({
        "contents": [{
            "parts": [{
                "text": prompt
            }]
        }]
    });

    let res = client.post(&url).json(&body).send().await?;

    if !res.status().is_success() {
        return Err(anyhow::anyhow!("Gemini API error: {}", res.status()));
    }

    let json: Value = res.json().await?;

    // Extract text from response
    let text = json["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .context("Failed to parse Gemini response text")?
        .to_string();

    Ok(text)
}

fn parse_response(
    files: Vec<PathBuf>,
    response_text: String,
    reason: &str,
) -> Result<Vec<RenameOp>> {
    // Clean up markdown code blocks if present
    let clean_text = response_text
        .trim()
        .trim_start_matches("```json")
        .trim_start_matches("```")
        .trim_end_matches("```");

    let mapping: std::collections::HashMap<String, String> = serde_json::from_str(clean_text)
        .context(format!("Failed to parse JSON from LLM: {}", clean_text))?;

    let mut ops = Vec::new();
    for file in files {
        let key = file.to_string_lossy().into_owned();
        // Also try matching just the filename if full path fails
        let name_key = file
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        if let Some(new_name) = mapping.get(&key).or_else(|| mapping.get(&name_key)) {
            // If new_name is just a filename, join it with parent
            let new_path = if new_name.contains('/') || new_name.contains('\\') {
                PathBuf::from(new_name)
            } else {
                file.parent().unwrap().join(new_name)
            };

            ops.push(RenameOp {
                original: file,
                new: new_path,
                reason: reason.to_string(),
            });
        }
    }
    Ok(ops)
}
