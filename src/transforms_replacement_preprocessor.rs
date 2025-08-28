/**
 * Allows us to use ${gitroot} and ${home} in replacement strings
 * TODO: refactor to allow easier generalization to allow other environment variables or placeholders
 */

// get git root from git command, in a Lazy way:
use std::sync::LazyLock;
use std::process::Command;
use log::debug;
use crate::path_util::home_dir;

static GIT_ROOT: LazyLock<String> = LazyLock::new(|| {
    let output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .expect("Failed to execute git command");

    if output.status.success() {
        String::from_utf8(output.stdout)
            .expect("Invalid UTF-8 from git command")
            .trim()
            .replace("\\", "\\\\") // TOOD: is this all the things we need to rpelace in
                                   // replacement strings???
            .replace("$", "\\$")
            .to_string()
    } else {
        debug!("Git command failed: {}", String::from_utf8_lossy(&output.stderr));
        "".to_string()
    }
});

// Replace ${gitroot} in replacement strings with the actual git root path, escaped
// This was done by AI, could likely be improved...
pub fn process_replacement_string(replacement: &str) -> String {
    let mut result = String::new();
    let mut chars = replacement.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '\\' => {
                if let Some('$') = chars.peek() {
                    let next = chars.next().unwrap();
                    if let Some('{') = chars.peek() {
                        result.push(next);
                        // Push the rest of the escaped sequence as a literal string
                        while let Some(next_c) = chars.next() {
                            result.push(next_c);
                            if next_c == '}' {
                                break;
                            }
                        }
                    } else {
                        // Not a ${...} sequence, so it's a regular escape
                        result.push(c);
                        result.push(next);
                    }
                } else {
                    result.push(c);
                }
            }
            '$' => {
                if let Some('{') = chars.peek() {
                    // Peek ahead to see if it's the full "${...}" pattern
                    let mut temp_chars = chars.clone();
                    let mut expansion_pattern = String::new();
                    
                    let mut _char_count = 0;
                    
                    while let Some(next_c) = temp_chars.next() {
                        expansion_pattern.push(next_c);
                        _char_count += 1;
                        if next_c == '}' {
                            break;
                        }
                    }
                    
                    if expansion_pattern.starts_with("{gitroot}") {
                        // Consume the characters we just peeked at
                        for _ in 0..expansion_pattern.len() {
                            chars.next();
                        }
                        result.push_str(GIT_ROOT.replace("\\", "\\\\").replace("$", "\\$").as_str());
                    } else if expansion_pattern.starts_with("{home}") {
                        for _ in 0..expansion_pattern.len() {
                            chars.next();
                        }
                        if let Ok(home_path) = home_dir() {
                            result.push_str(home_path.display().to_string().replace("\\", "\\\\").replace("$", "\\$").as_str());
                        }
                    } else {
                        result.push(c);
                    }
                } else {
                    result.push(c);
                }
            }
            _ => result.push(c),
        }
    }
    result
}


