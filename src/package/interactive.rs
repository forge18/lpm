use crate::core::{LpmError, LpmResult};
use std::io::{self, Write};

/// Prompt the user for confirmation
pub fn confirm(prompt: &str) -> LpmResult<bool> {
    print!("{} (y/N): ", prompt);
    io::stdout().flush().map_err(|e| {
        LpmError::Package(format!("Failed to write to stdout: {}", e))
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| LpmError::Package(format!("Failed to read from stdin: {}", e)))?;

    let trimmed = input.trim().to_lowercase();
    Ok(trimmed == "y" || trimmed == "yes")
}

/// Prompt the user with a default value
pub fn confirm_with_default(prompt: &str, default: bool) -> LpmResult<bool> {
    let default_str = if default { "Y/n" } else { "y/N" };
    print!("{} ({}): ", prompt, default_str);
    io::stdout().flush().map_err(|e| {
        LpmError::Package(format!("Failed to write to stdout: {}", e))
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| LpmError::Package(format!("Failed to read from stdin: {}", e)))?;

    let trimmed = input.trim().to_lowercase();
    
    if trimmed.is_empty() {
        Ok(default)
    } else {
        Ok(trimmed == "y" || trimmed == "yes")
    }
}

/// Prompt for a choice from a list of options
pub fn choose(prompt: &str, options: &[&str], default: usize) -> LpmResult<usize> {
    println!("{}", prompt);
    for (i, option) in options.iter().enumerate() {
        let marker = if i == default { "*" } else { " " };
        println!("  {}[{}] {}", marker, i + 1, option);
    }

    print!("Choose (1-{}, default {}): ", options.len(), default + 1);
    io::stdout().flush().map_err(|e| {
        LpmError::Package(format!("Failed to write to stdout: {}", e))
    })?;

    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| LpmError::Package(format!("Failed to read from stdin: {}", e)))?;

    let trimmed = input.trim();
    
    if trimmed.is_empty() {
        Ok(default)
    } else {
        match trimmed.parse::<usize>() {
            Ok(n) if n >= 1 && n <= options.len() => Ok(n - 1),
            _ => {
                println!("Invalid choice, using default: {}", default + 1);
                Ok(default)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Note: These tests would require mocking stdin/stdout
    // For now, we just test that the functions compile
    #[test]
    fn test_confirm_function_exists() {
        // This test just ensures the function signature is correct
        let _ = confirm;
    }
}

