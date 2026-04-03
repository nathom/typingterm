use rand::seq::SliceRandom;
use rand::Rng;
use std::collections::HashMap;
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/languages_generated.rs"));

const QUOTES: &str = include_str!("../data/quotes.txt");

static LANG_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
static CODE_MAP: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();

fn lang_map() -> &'static HashMap<&'static str, &'static str> {
    LANG_MAP.get_or_init(build_language_map)
}

fn code_map() -> &'static HashMap<&'static str, &'static str> {
    CODE_MAP.get_or_init(build_code_map)
}

/// All available language names (natural + code)
pub fn all_language_names() -> Vec<&'static str> {
    let mut names: Vec<&str> = LANGUAGE_NAMES.to_vec();
    names.extend_from_slice(CODE_LANGUAGE_NAMES);
    names
}

/// All code language names
pub fn code_language_names() -> &'static [&'static str] {
    CODE_LANGUAGE_NAMES
}

pub fn is_code_language(name: &str) -> bool {
    name.starts_with("code ")
}

fn get_word_list(language: &str) -> Vec<&'static str> {
    if let Some(raw) = lang_map().get(language) {
        raw.lines().filter(|l| !l.is_empty()).collect()
    } else {
        Vec::new()
    }
}

/// Generate words for a typing test, matching monkeytype's algorithm:
/// - Uniform random selection from word list
/// - No consecutive repeats (checks last 2 words)
/// - Up to 100 retries per word to avoid repeats
pub fn generate_words(
    language: &str,
    count: usize,
    punctuation: bool,
    numbers: bool,
) -> Vec<String> {
    let word_list = get_word_list(language);
    if word_list.is_empty() {
        return vec!["error loading words".to_string()];
    }

    let mut rng = rand::thread_rng();
    let mut words: Vec<String> = Vec::with_capacity(count);
    let mut prev1 = String::new();
    let mut prev2 = String::new();
    let mut after_sentence_end = true;

    for i in 0..count {
        if numbers && rng.gen_ratio(1, 10) {
            let num: u32 = rng.gen_range(0..10000);
            let num_str = num.to_string();
            prev2 = prev1.clone();
            prev1 = num_str.clone();
            words.push(num_str);
            after_sentence_end = false;
            continue;
        }

        let mut word = String::new();
        for _ in 0..100 {
            let candidate = word_list.choose(&mut rng).unwrap().to_lowercase();
            if candidate == "i" {
                continue;
            }
            if candidate != prev1 && candidate != prev2 {
                word = candidate;
                break;
            }
        }
        if word.is_empty() {
            word = word_list.choose(&mut rng).unwrap().to_lowercase();
        }

        if punctuation {
            word = apply_punctuation(&word, &mut after_sentence_end, i, count, &mut rng);
        }

        prev2 = prev1.clone();
        prev1 = word
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect::<String>()
            .to_lowercase();
        words.push(word);
    }

    words
}

fn apply_punctuation(
    word: &str,
    after_sentence_end: &mut bool,
    index: usize,
    total: usize,
    rng: &mut impl Rng,
) -> String {
    let mut result = word.to_string();

    if *after_sentence_end {
        result = capitalize(&result);
        *after_sentence_end = false;
    }

    if index >= total - 1 {
        return format!("{}.", result);
    }

    let roll: f64 = rng.gen();

    if roll < 0.06 {
        result = format!("{},", result);
    } else if roll < 0.09 {
        result = format!("{}.", result);
        *after_sentence_end = true;
    } else if roll < 0.11 {
        result = format!("{}?", result);
        *after_sentence_end = true;
    } else if roll < 0.12 {
        result = format!("{}!", result);
        *after_sentence_end = true;
    } else if roll < 0.14 {
        result = format!("{};", result);
    } else if roll < 0.16 {
        result = format!("{}:", result);
    } else if roll < 0.18 {
        result = format!("\"{}\"", result);
    }

    result
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().to_string() + c.as_str(),
    }
}

/// Get a random code snippet for the given code language.
pub fn get_code_snippet(language: &str) -> Vec<String> {
    let data = match code_map().get(language) {
        Some(d) => *d,
        None => return vec!["// no code snippets available".to_string()],
    };

    let snippets: Vec<&str> = data.split("\n---\n").filter(|s| !s.trim().is_empty()).collect();
    let mut rng = rand::thread_rng();
    let snippet = snippets.choose(&mut rng).unwrap_or(&"// error");

    snippet
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.trim().is_empty())
        .collect()
}

/// Quote: (source, text)
pub fn get_random_quote() -> (String, String) {
    let quotes: Vec<&str> = QUOTES.lines().filter(|l| !l.is_empty()).collect();
    let mut rng = rand::thread_rng();
    let line = quotes
        .choose(&mut rng)
        .unwrap_or(&"Unknown\tThe quick brown fox jumps over the lazy dog.");
    let mut parts = line.splitn(2, '\t');
    let source = parts.next().unwrap_or("Unknown").to_string();
    let text = parts
        .next()
        .unwrap_or("The quick brown fox jumps over the lazy dog.")
        .to_string();
    (source, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_words_basic() {
        let words = generate_words("english", 25, false, false);
        assert_eq!(words.len(), 25);
        for w in &words {
            assert!(!w.is_empty());
        }
    }

    #[test]
    fn test_no_consecutive_repeats() {
        let words = generate_words("english", 200, false, false);
        for i in 1..words.len() {
            assert_ne!(
                words[i].to_lowercase(),
                words[i - 1].to_lowercase(),
                "Consecutive repeat at index {}: '{}'",
                i,
                words[i]
            );
        }
    }

    #[test]
    fn test_generate_words_with_punctuation() {
        let words = generate_words("english", 100, true, false);
        assert_eq!(words.len(), 100);
        let first_char = words[0].chars().next().unwrap();
        assert!(first_char.is_uppercase() || first_char == '"');
    }

    #[test]
    fn test_generate_words_with_numbers() {
        let words = generate_words("english", 200, false, true);
        assert_eq!(words.len(), 200);
        let has_number = words
            .iter()
            .any(|w| w.chars().next().map_or(false, |c| c.is_ascii_digit()));
        assert!(has_number, "Expected at least one number in 200 words");
    }

    #[test]
    fn test_many_languages_load() {
        for lang in ["english", "english_1k", "french", "german", "spanish", "turkish", "japanese_romaji", "korean"] {
            let words = get_word_list(lang);
            assert!(!words.is_empty(), "Language '{}' has no words", lang);
        }
    }

    #[test]
    fn test_code_snippets_load() {
        for lang in code_language_names() {
            let lines = get_code_snippet(lang);
            assert!(!lines.is_empty(), "Code language '{}' has no snippets", lang);
            assert!(
                lines[0] != "// no code snippets available",
                "No snippets for '{}'",
                lang
            );
        }
    }

    #[test]
    fn test_all_language_names() {
        let names = all_language_names();
        assert!(names.len() > 100, "Expected 100+ languages, got {}", names.len());
        assert!(names.contains(&"english"));
        assert!(names.contains(&"code rust"));
    }

    #[test]
    fn test_get_random_quote() {
        let (source, text) = get_random_quote();
        assert!(!source.is_empty());
        assert!(!text.is_empty());
    }
}
