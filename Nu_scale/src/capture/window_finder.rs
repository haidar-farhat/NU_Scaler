use anyhow::{Result, anyhow};
use std::collections::HashMap;

use super::{CaptureError, platform::WindowInfo};

/// Simple fuzzy matching algorithm for window titles
fn fuzzy_match(haystack: &str, needle: &str) -> bool {
    let haystack = haystack.to_lowercase();
    let needle = needle.to_lowercase();
    
    haystack.contains(&needle)
}

/// Calculates similarity score between window title and search pattern
fn similarity_score(window_title: &str, search_pattern: &str) -> f32 {
    let title_lower = window_title.to_lowercase();
    let pattern_lower = search_pattern.to_lowercase();
    
    if title_lower == pattern_lower {
        // Exact match
        return 1.0;
    }
    
    if title_lower.contains(&pattern_lower) {
        // Contains the pattern
        let position = title_lower.find(&pattern_lower).unwrap() as f32;
        let title_len = title_lower.len() as f32;
        
        // Higher score for matches at the beginning
        return 0.8 - (position / title_len) * 0.3;
    }
    
    // Check for partial word matches
    let title_words: Vec<&str> = title_lower.split_whitespace().collect();
    let pattern_words: Vec<&str> = pattern_lower.split_whitespace().collect();
    
    let mut matched_words = 0;
    for pattern_word in &pattern_words {
        for title_word in &title_words {
            if title_word.contains(pattern_word) {
                matched_words += 1;
                break;
            }
        }
    }
    
    if matched_words > 0 {
        return 0.4 * (matched_words as f32 / pattern_words.len() as f32);
    }
    
    // No match
    0.0
}

/// Finds the best matching window for a search pattern
pub fn find_best_match(windows: &[WindowInfo], search_pattern: &str) -> Result<WindowInfo> {
    if search_pattern.is_empty() {
        return Err(anyhow!(CaptureError::InvalidParameters));
    }
    
    let mut best_match = None;
    let mut best_score = 0.0;
    
    for window in windows {
        let score = similarity_score(&window.title, search_pattern);
        if score > best_score {
            best_score = score;
            best_match = Some(window);
        }
    }
    
    best_match.cloned().ok_or_else(|| anyhow!(CaptureError::WindowNotFound))
}

/// Finds all windows matching a search pattern with scores
pub fn find_matching_windows(windows: &[WindowInfo], search_pattern: &str) -> Vec<(WindowInfo, f32)> {
    if search_pattern.is_empty() {
        return Vec::new();
    }
    
    let mut matches: Vec<(WindowInfo, f32)> = Vec::new();
    
    for window in windows {
        let score = similarity_score(&window.title, search_pattern);
        if score > 0.0 {
            matches.push((window.clone(), score));
        }
    }
    
    // Sort by score (descending)
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    matches
}

/// Finds a window by an exact title match
pub fn find_by_exact_title(windows: &[WindowInfo], title: &str) -> Result<WindowInfo> {
    let title_lower = title.to_lowercase();
    
    for window in windows {
        if window.title.to_lowercase() == title_lower {
            return Ok(window.clone());
        }
    }
    
    Err(anyhow!(CaptureError::WindowNotFound))
}

/// Groups windows by application class
pub fn group_by_application(windows: &[WindowInfo]) -> HashMap<String, Vec<WindowInfo>> {
    let mut groups: HashMap<String, Vec<WindowInfo>> = HashMap::new();
    
    for window in windows {
        if let Some(class) = &window.class {
            groups.entry(class.clone()).or_default().push(window.clone());
        } else {
            // Group by title if class is not available
            groups.entry(window.title.clone()).or_default().push(window.clone());
        }
    }
    
    groups
} 