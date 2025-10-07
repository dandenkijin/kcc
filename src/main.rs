use std::env;
use std::fs;
use std::path::Path;
use clap::Parser;
use colored::*;

#[derive(Parser)]
#[command(name = "kcc", author, version, about, long_about = None)]
struct Cli {
    /// Path to kernel config file (default: /proc/config.gz)
    #[arg(short, long, default_value = "/proc/config.gz")]
    config: String,

    /// Path to flags file containing kernel config flags to check
    #[arg(short, long, value_name = "FILE")]
    flags: Vec<String>,

    /// Disable colored output
    #[arg(short, long)]
    no_color: bool,
}

#[derive(Debug, PartialEq)]
enum FlagStatus {
    EnabledInKernel,
    EnabledAsModule,
    Missing,
}

struct FlagCheckResult {
    name: String,
    status: FlagStatus,
}

impl FlagCheckResult {
    fn format_output(&self) -> String {
        if self.status == FlagStatus::EnabledInKernel {
            format!("✅ {}", self.name.green())
        } else if self.status == FlagStatus::EnabledAsModule {
            format!("✅ {} (as module)", self.name.green())
        } else {
            format!("❌ {}", self.name.red())
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up color output
    if cli.no_color {
        colored::control::set_override(true);
    }

    if cli.flags.is_empty() {
        return Err(anyhow::anyhow!("At least one flags file must be specified with -f or --flags"));
    }

    let config_content = read_kernel_config(&cli.config)?;
    let mut all_flags = Vec::new();
    
    for flag_file in &cli.flags {
        let flags = read_flags_file(flag_file)?;
        all_flags.extend(flags);
    }

    println!("🔍 Kernel Config Checker - Checking kernel configuration flags from: {}", cli.config);
    println!("📋 Reading flags from files: {}", cli.flags.join(", "));
    println!();

    let mut exit_code = 0;

    for flag in all_flags {
        let result = check_flag(&config_content, &flag);
        println!("{}", result.format_output());
        
        if result.status == FlagStatus::Missing {
            exit_code = 1;
        }
    }

    println!();
    if exit_code == 0 {
        println!("✅ All required kernel flags are enabled!");
    } else {
        println!("❌ Some required kernel flags are missing!");
    }

    std::process::exit(exit_code);
}

fn read_kernel_config(path: &str) -> anyhow::Result<String> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Config file not found: {}", path.display()));
    }

    if path.extension().and_then(|s| s.to_str()) == Some("gz") {
        // Handle compressed config files
        use std::process::Command;
        let output = Command::new("zcat")
            .arg(path)
            .output()
            .map_err(|e| anyhow::anyhow!("Failed to run zcat: {}", e))?;
        
        if !output.status.success() {
            return Err(anyhow::anyhow!("zcat failed: {}", String::from_utf8_lossy(&output.stderr)));
        }
        
        Ok(String::from_utf8(output.stdout)?)
    } else {
        // Handle uncompressed config files
        fs::read_to_string(path).map_err(|e| anyhow::anyhow!("Failed to read config file: {}", e))
    }
}

fn read_flags_file(path: &str) -> anyhow::Result<Vec<String>> {
    let content = fs::read_to_string(path)
        .map_err(|e| anyhow::anyhow!("Failed to read flags file: {}", e))?;
    
    let mut flags = Vec::new();
    
    for line in content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        
        // Handle both "FLAG" and "FLAG=value" formats
        if trimmed.contains('=') {
            // Extract the flag name before the equals sign
            let flag_name = trimmed.split_once('=').unwrap().0;
            flags.push(flag_name.to_string());
        } else {
            // It's just a flag name
            flags.push(trimmed.to_string());
        }
    }
    
    Ok(flags)
}

fn check_flag(config_content: &str, flag: &str) -> FlagCheckResult {
    // Remove CONFIG_ prefix if it already exists in the input
    let clean_flag = if flag.starts_with("CONFIG_") {
        &flag[7..]
    } else {
        flag
    };
    
    let config_flag = format!("CONFIG_{}=", clean_flag);
    
    for line in config_content.lines() {
        if line.starts_with(&config_flag) {
            let value = &line[config_flag.len()..];
            match value {
                "y" => return FlagCheckResult {
                    name: format!("CONFIG_{}", clean_flag),
                    status: FlagStatus::EnabledInKernel,
                },
                "m" => return FlagCheckResult {
                    name: format!("CONFIG_{}", clean_flag),
                    status: FlagStatus::EnabledAsModule,
                },
                _ => {}
            }
        }
    }
    
    FlagCheckResult {
        name: format!("CONFIG_{}", clean_flag),
        status: FlagStatus::Missing,
    }
}