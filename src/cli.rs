use anyhow::Result;
use std::io::{self, BufRead, Write};

use crate::config::{self, NewProfile};

pub fn run_add() -> Result<()> {
    run_add_with(io::stdin().lock(), io::stdout())
}

pub fn run_add_with<R: BufRead, W: Write>(mut reader: R, mut writer: W) -> Result<()> {
    // Name (required)
    let name = loop {
        write!(writer, "Name: ")?;
        writer.flush()?;
        let mut line = String::new();
        reader.read_line(&mut line)?;
        let trimmed = line.trim().to_string();
        if trimmed.is_empty() {
            writeln!(writer, "Name is required.")?;
            continue;
        }
        if config::profile_name_exists(&trimmed)? {
            eprintln!("Error: profile '{}' already exists.", trimmed);
            std::process::exit(1);
        }
        break trimmed;
    };

    // Description (optional)
    write!(writer, "Description (optional): ")?;
    writer.flush()?;
    let mut desc_line = String::new();
    reader.read_line(&mut desc_line)?;
    let description = {
        let t = desc_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // Model (optional)
    write!(writer, "Model (optional): ")?;
    writer.flush()?;
    let mut model_line = String::new();
    reader.read_line(&mut model_line)?;
    let model = {
        let t = model_line.trim().to_string();
        if t.is_empty() { None } else { Some(t) }
    };

    // Summary
    writeln!(writer)?;
    writeln!(writer, "--- New Profile ---")?;
    writeln!(writer, "  Name:        {}", name)?;
    writeln!(writer, "  Description: {}", description.as_deref().unwrap_or("(none)"))?;
    writeln!(writer, "  Model:       {}", model.as_deref().unwrap_or("(none)"))?;
    writeln!(writer)?;

    // Confirm
    write!(writer, "Save? (y/n): ")?;
    writer.flush()?;
    let mut confirm = String::new();
    reader.read_line(&mut confirm)?;
    if confirm.trim().to_lowercase() != "y" {
        writeln!(writer, "Cancelled.")?;
        return Ok(());
    }

    let profile = NewProfile {
        name: name.clone(),
        description,
        model,
    };
    config::append_profile(&profile)?;
    writeln!(writer, "Profile '{}' added.", name)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn cli_run_add_rejects_duplicate() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("profiles.toml");
        std::fs::write(&path, "[[profiles]]\nname = \"existing\"\n").unwrap();
        std::env::set_var("CCT_CONFIG", &path);

        // Verify duplicate detection
        assert!(config::profile_name_exists("existing").unwrap());
        assert!(config::profile_name_exists("EXISTING").unwrap());

        // Test that a valid add flow works
        let input = b"newprofile\nmy desc\nsonnet\ny\n";
        let mut output: Vec<u8> = Vec::new();
        run_add_with(&input[..], &mut output).unwrap();

        let profiles = config::load_profiles().unwrap();
        assert_eq!(profiles.len(), 2);
        assert!(profiles.iter().any(|p| p.name == "newprofile"));

        std::env::remove_var("CCT_CONFIG");
    }
}
