use crate::config::Language;
use rand::seq::SliceRandom;
use rand::Rng;

// Embed all word lists at compile time
const ENGLISH: &str = include_str!("../data/languages/english.txt");
const ENGLISH_1K: &str = include_str!("../data/languages/english_1k.txt");
const ENGLISH_5K: &str = include_str!("../data/languages/english_5k.txt");
const ENGLISH_10K: &str = include_str!("../data/languages/english_10k.txt");
const SPANISH: &str = include_str!("../data/languages/spanish.txt");
const FRENCH: &str = include_str!("../data/languages/french.txt");
const GERMAN: &str = include_str!("../data/languages/german.txt");
const PORTUGUESE: &str = include_str!("../data/languages/portuguese.txt");
const ITALIAN: &str = include_str!("../data/languages/italian.txt");
const DUTCH: &str = include_str!("../data/languages/dutch.txt");
const SWEDISH: &str = include_str!("../data/languages/swedish.txt");

const CODE_RUST: &str = include_str!("../data/code/rust.txt");
const CODE_PYTHON: &str = include_str!("../data/code/python.txt");
const CODE_JAVASCRIPT: &str = include_str!("../data/code/javascript.txt");
const CODE_TYPESCRIPT: &str = include_str!("../data/code/typescript.txt");
const CODE_GO: &str = include_str!("../data/code/go.txt");
const CODE_C: &str = include_str!("../data/code/c.txt");
const CODE_CPP: &str = include_str!("../data/code/cpp.txt");
const CODE_JAVA: &str = include_str!("../data/code/java.txt");

const QUOTES: &str = include_str!("../data/quotes.txt");

fn get_word_list(language: Language) -> Vec<&'static str> {
    let raw = match language {
        Language::English => ENGLISH,
        Language::English1k => ENGLISH_1K,
        Language::English5k => ENGLISH_5K,
        Language::English10k => ENGLISH_10K,
        Language::Spanish => SPANISH,
        Language::French => FRENCH,
        Language::German => GERMAN,
        Language::Portuguese => PORTUGUESE,
        Language::Italian => ITALIAN,
        Language::Dutch => DUTCH,
        Language::Swedish => SWEDISH,
        // Code languages return empty here; use get_code_snippet instead
        _ => "",
    };
    raw.lines().filter(|l| !l.is_empty()).collect()
}

fn get_code_data(language: Language) -> &'static str {
    match language {
        Language::CodeRust => CODE_RUST,
        Language::CodePython => CODE_PYTHON,
        Language::CodeJavascript => CODE_JAVASCRIPT,
        Language::CodeTypescript => CODE_TYPESCRIPT,
        Language::CodeGo => CODE_GO,
        Language::CodeC => CODE_C,
        Language::CodeCpp => CODE_CPP,
        Language::CodeJava => CODE_JAVA,
        _ => "",
    }
}

/// Generate words for a typing test, matching monkeytype's algorithm:
/// - Uniform random selection from word list
/// - No consecutive repeats (checks last 2 words)
/// - Up to 100 retries per word to avoid repeats
pub fn generate_words(
    language: Language,
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
    let mut prev1 = String::new(); // previous word (lowercase, no punctuation)
    let mut prev2 = String::new(); // word before that
    let mut after_sentence_end = true; // capitalize first word and after periods

    for i in 0..count {
        // ~10% chance of a number if numbers enabled
        if numbers && rng.gen_ratio(1, 10) {
            // Generate a random 1-4 digit number like monkeytype
            let num: u32 = rng.gen_range(0..10000);
            let num_str = num.to_string();
            prev2 = prev1.clone();
            prev1 = num_str.clone();
            words.push(num_str);
            after_sentence_end = false;
            continue;
        }

        // Pick a word, avoiding repeats of last 2 words (monkeytype algorithm)
        let mut word = String::new();
        for _ in 0..100 {
            let candidate = word_list.choose(&mut rng).unwrap().to_lowercase();
            // Skip "I" (monkeytype filters this out)
            if candidate == "i" {
                continue;
            }
            if candidate != prev1 && candidate != prev2 {
                word = candidate;
                break;
            }
        }
        if word.is_empty() {
            // Fallback if all retries failed
            word = word_list.choose(&mut rng).unwrap().to_lowercase();
        }

        // Apply punctuation
        if punctuation {
            word = apply_punctuation(&word, &mut after_sentence_end, i, count, &mut rng);
        }

        prev2 = prev1.clone();
        // Store normalized version for comparison (strip punctuation)
        prev1 = word.chars().filter(|c| c.is_alphanumeric()).collect::<String>().to_lowercase();
        words.push(word);
    }

    words
}

/// Apply punctuation following monkeytype's probabilistic rules:
/// - Capitalize after sentence-ending punctuation
/// - Commas, periods, question marks, exclamation marks with realistic probabilities
fn apply_punctuation(
    word: &str,
    after_sentence_end: &mut bool,
    index: usize,
    total: usize,
    rng: &mut impl Rng,
) -> String {
    let mut result = word.to_string();

    // Capitalize if this is after a sentence end (or first word)
    if *after_sentence_end {
        result = capitalize(&result);
        *after_sentence_end = false;
    }

    // Don't add punctuation to last word (monkeytype doesn't)
    if index >= total - 1 {
        // End with a period
        return format!("{}.", result);
    }

    let roll: f64 = rng.gen();

    if roll < 0.06 {
        // Comma
        result = format!("{},", result);
    } else if roll < 0.09 {
        // Period - ends sentence
        result = format!("{}.", result);
        *after_sentence_end = true;
    } else if roll < 0.11 {
        // Question mark
        result = format!("{}?", result);
        *after_sentence_end = true;
    } else if roll < 0.12 {
        // Exclamation mark
        result = format!("{}!", result);
        *after_sentence_end = true;
    } else if roll < 0.14 {
        // Semicolon
        result = format!("{};", result);
    } else if roll < 0.16 {
        // Colon
        result = format!("{}:", result);
    } else if roll < 0.18 {
        // Quoted
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
/// Returns the lines of code as a Vec<String>.
/// Leading indentation is preserved in the lines (the typing engine
/// will auto-insert it so the user doesn't have to type it).
pub fn get_code_snippet(language: Language) -> Vec<String> {
    let data = get_code_data(language);
    if data.is_empty() {
        return vec!["// no code snippets available".to_string()];
    }

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
    let line = quotes.choose(&mut rng).unwrap_or(&"Unknown\tThe quick brown fox jumps over the lazy dog.");
    let mut parts = line.splitn(2, '\t');
    let source = parts.next().unwrap_or("Unknown").to_string();
    let text = parts.next().unwrap_or("The quick brown fox jumps over the lazy dog.").to_string();
    (source, text)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_words_basic() {
        let words = generate_words(Language::English, 25, false, false);
        assert_eq!(words.len(), 25);
        for w in &words {
            assert!(!w.is_empty());
        }
    }

    #[test]
    fn test_no_consecutive_repeats() {
        // Generate a large batch and check no consecutive repeats
        let words = generate_words(Language::English, 200, false, false);
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
        let words = generate_words(Language::English, 100, true, false);
        assert_eq!(words.len(), 100);
        // First word should be capitalized
        let first_char = words[0].chars().next().unwrap();
        assert!(first_char.is_uppercase() || first_char == '"');
    }

    #[test]
    fn test_generate_words_with_numbers() {
        let words = generate_words(Language::English, 200, false, true);
        assert_eq!(words.len(), 200);
        // Should have at least some numbers with 200 words at 10% rate
        let has_number = words.iter().any(|w| w.chars().next().map_or(false, |c| c.is_ascii_digit()));
        assert!(has_number, "Expected at least one number in 200 words");
    }

    #[test]
    fn test_natural_languages_load() {
        for lang in Language::natural_languages() {
            let words = get_word_list(*lang);
            assert!(!words.is_empty(), "Language {:?} has no words", lang);
        }
    }

    #[test]
    fn test_code_snippets_load() {
        for lang in Language::code_languages() {
            let lines = get_code_snippet(*lang);
            assert!(!lines.is_empty(), "Code language {:?} has no snippets", lang);
            assert!(lines[0] != "// no code snippets available", "No snippets for {:?}", lang);
        }
    }

    #[test]
    fn test_get_random_quote() {
        let (source, text) = get_random_quote();
        assert!(!source.is_empty());
        assert!(!text.is_empty());
    }
}
