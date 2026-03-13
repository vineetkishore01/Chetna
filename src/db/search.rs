//! Search utilities

use crate::db::brain::Memory;

/// Advanced search with weighted scoring
pub fn rank_memories(memories: Vec<Memory>, query: &str) -> Vec<Memory> {
    let query_lower = query.to_lowercase();
    let query_words: Vec<&str> = query_lower.split_whitespace().collect();
    
    let mut scored: Vec<(Memory, f64)> = memories
        .into_iter()
        .map(|m| {
            let mut score = m.importance;
            
            // Boost for exact content match
            if m.content.to_lowercase().contains(&query_lower) {
                score += 0.3;
            }
            
            // Boost for keyword matches
            for word in &query_words {
                if m.content.to_lowercase().contains(word) {
                    score += 0.1;
                }
                if let Some(ref key) = m.key {
                    if key.to_lowercase().contains(word) {
                        score += 0.2;
                    }
                }
            }
            
            // Boost for access count (frequently accessed = relevant)
            score += (m.access_count as f64 * 0.01).min(0.2);
            
            (m, score)
        })
        .collect();
    
    // Sort by score descending
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    
    scored.into_iter().map(|(m, _)| m).collect()
}

/// Extract keywords from query for fuzzy matching
pub fn extract_keywords(query: &str) -> Vec<String> {
    let query_lower = query.to_lowercase();
    let words: Vec<String> = query_lower
        .split(|c: char| !c.is_alphanumeric())
        .filter(|s| s.len() > 2)
        .map(|s| s.to_string())
        .collect();
    
    // Remove common stop words
    let stop_words = ["the", "and", "for", "that", "this", "with", "from", "have", "will", "can", "are", "was", "were"];
    
    words.into_iter()
        .filter(|w| !stop_words.contains(&w.as_str()))
        .collect()
}
