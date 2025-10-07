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

    /// Specific kernel config flags to check (comma-separated)
    #[arg(long, value_name = "FLAGS")]
    set_flags: Vec<String>,

    /// Set flags from file (adds missing flags to .config file)
    #[arg(long)]
    set: bool,

    /// Disable colored output
    #[arg(short, long)]
    no_color: bool,

    /// Check for flags in the list that are missing from config
    #[arg(long)]
    check_incomplete: bool,

    /// Show only missing flags
    #[arg(long)]
    check_missing: bool,
}

#[derive(Debug, PartialEq)]
enum FlagStatus {
    EnabledInKernel,
    EnabledAsModule,
    Missing,
    Invalid, // Flag doesn't exist in kernel config options
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
        } else if self.status == FlagStatus::Missing {
            format!("❌ {}", self.name.red())
        } else {
            format!("⚠️  {} (invalid flag)", self.name.yellow())
        }
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Set up color output
    if cli.no_color {
        colored::control::set_override(true);
    }

    if cli.flags.is_empty() && cli.set_flags.is_empty() {
        return Err(anyhow::anyhow!("At least one flags file or set flags must be specified with -f/--flags or --set-flags"));
    }

    if cli.set {
        return set_kernel_config_flags(&cli.config, &cli.flags, &cli.set_flags);
    }

    let config_content = read_kernel_config(&cli.config)?;
    let mut all_flags = Vec::new();
    
    // Read flags from files
    for flag_file in &cli.flags {
        let flags = read_flags_file(flag_file)?;
        all_flags.extend(flags);
    }
    
    // Add directly set flags (handle comma-separated values)
    for flags_str in &cli.set_flags {
        let flags: Vec<String> = flags_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        all_flags.extend(flags);
    }

    println!("🔍 Kernel Config Checker - Checking kernel configuration flags from: {}", cli.config);
    if !cli.flags.is_empty() {
        println!("📋 Reading flags from files: {}", cli.flags.join(", "));
    }
    if !cli.set_flags.is_empty() {
        println!("📋 Checking specified flags: {}", cli.set_flags.join(", "));
    }
    println!();

    let mut exit_code = 0;
    let mut missing_flags_in_list = Vec::new();
    let mut invalid_flags_in_list = Vec::new();

    for flag in &all_flags {
        let result = check_flag(&config_content, flag);
        println!("{}", result.format_output());
        
        if result.status == FlagStatus::Missing {
            exit_code = 1;
            missing_flags_in_list.push(result.name);
        } else if result.status == FlagStatus::Invalid {
            exit_code = 1;
            invalid_flags_in_list.push(result.name);
        }
    }

    // Check for issues with flags in the list
    if !missing_flags_in_list.is_empty() || !invalid_flags_in_list.is_empty() {
        println!();
        if !missing_flags_in_list.is_empty() {
            println!("⚠️  Flags in your list that are missing from config:");
            for flag in &missing_flags_in_list {
                println!("   - {}", flag.red());
            }
        }
        if !invalid_flags_in_list.is_empty() {
            println!("⚠️  Flags in your list that don't exist in kernel config options:");
            for flag in &invalid_flags_in_list {
                println!("   - {}", flag.yellow());
            }
        }
        if !missing_flags_in_list.is_empty() {
            println!("📝 Consider using --set to add missing flags to your config file");
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

fn check_kernel_config_exists(flag: &str) -> bool {
    let path = "/proc/config.gz";
    
    if let Ok(config_content) = read_kernel_config(path) {
        for line in config_content.lines() {
            if line.starts_with(flag) {
                return true;
            }
        }
    }
    
    // Fallback: try to read directly with zcat
    use std::process::Command;
    let output = Command::new("zcat")
        .arg(path)
        .output();
    
    if let Ok(output) = output {
        if output.status.success() {
            let config_content = String::from_utf8_lossy(&output.stdout);
            for line in config_content.lines() {
                if line.starts_with(flag) {
                    return true;
                }
            }
        }
    }
    
    false
}

fn check_flag(config_content: &str, flag: &str) -> FlagCheckResult {
    // Remove CONFIG_ prefix if it already exists in the input
    let clean_flag = if flag.starts_with("CONFIG_") {
        &flag[7..]
    } else {
        flag
    };
    
    let config_flag = format!("CONFIG_{}=", clean_flag);
    
    // Check if the flag actually exists in kernel config options
    if !check_kernel_config_exists(&config_flag) {
        return FlagCheckResult {
            name: format!("CONFIG_{}", clean_flag),
            status: FlagStatus::Invalid,
        };
    }
    
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

fn set_kernel_config_flags(config_path: &str, flag_files: &[String], set_flags: &[String]) -> anyhow::Result<()> {
    println!("🔧 Adding flags to kernel config file: {}", config_path);
    
    let mut all_flags = Vec::new();
    
    // Read flags from files
    for flag_file in flag_files {
        let flags = read_flags_file(flag_file)?;
        all_flags.extend(flags);
        println!("📋 Reading flags from file: {}", flag_file);
    }
    
    // Add directly set flags (handle comma-separated values)
    for flags_str in set_flags {
        let flags: Vec<String> = flags_str.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        all_flags.extend(flags);
        println!("📋 Adding specified flags: {}", flags_str);
    }

    // Remove duplicates
    all_flags.sort();
    all_flags.dedup();

    println!();
    println!("🎯 Adding {} flags to .config file:", all_flags.len());

    // Read the current config file
    let config_content = read_kernel_config(config_path)?;
    let mut config_lines: Vec<String> = config_content.lines().map(|s| s.to_string()).collect();
    let mut added_count = 0;
    let mut already_exists_count = 0;

    for flag in &all_flags {
        let clean_flag = if flag.starts_with("CONFIG_") {
            &flag[7..]
        } else {
            flag
        };
        
        let config_flag = format!("CONFIG_{}=", clean_flag);
        let config_line = format!("CONFIG_{}=y", clean_flag);
        
        // Check if flag already exists
        let flag_exists = config_lines.iter().any(|line| line.starts_with(&config_flag));
        
        if flag_exists {
            println!("⚠️  {}: already exists", config_flag.yellow());
            already_exists_count += 1;
        } else {
            // Add the flag to the config content at the end
            config_lines.push(config_line);
            println!("✅ {}: ADDED", config_flag.green());
            added_count += 1;
        }
    }

    // Join lines with proper newlines and write back
    let updated_config = config_lines.join("\n") + "\n";
    fs::write(config_path, &updated_config)?;

    println!();
    if added_count > 0 {
        println!("✅ Successfully added {} flags to .config file!", added_count);
    }
    if already_exists_count > 0 {
        println!("ℹ️  {} flags already existed and were not modified.", already_exists_count);
    }

    Ok(())
}