use std::time::Instant;

/// State of each character in the test
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CharState {
    Pending,
    Correct,
    Incorrect,
    Extra,
}

/// A single word (or code line) in the test
#[derive(Debug, Clone)]
pub struct Word {
    pub target: Vec<char>,
    pub states: Vec<CharState>,
    pub typed: Vec<char>,
    pub completed: bool,
}

impl Word {
    pub fn new(text: &str) -> Self {
        let target: Vec<char> = text.chars().collect();
        let len = target.len();
        Self {
            target,
            states: vec![CharState::Pending; len],
            typed: Vec::new(),
            completed: false,
        }
    }

    pub fn cursor_pos(&self) -> usize {
        self.typed.len()
    }

    pub fn is_correct(&self) -> bool {
        self.typed.len() == self.target.len()
            && self.states.iter().all(|s| *s == CharState::Correct)
    }

    pub fn type_char(&mut self, c: char) {
        let pos = self.typed.len();
        self.typed.push(c);

        if pos < self.target.len() {
            if c == self.target[pos] {
                self.states[pos] = CharState::Correct;
            } else {
                self.states[pos] = CharState::Incorrect;
            }
        } else {
            self.states.push(CharState::Extra);
        }
    }

    pub fn backspace(&mut self) {
        if self.typed.is_empty() {
            return;
        }
        let pos = self.typed.len() - 1;
        self.typed.pop();
        if pos < self.target.len() {
            self.states[pos] = CharState::Pending;
        } else {
            self.states.pop();
        }
    }

    pub fn finalize(&mut self) {
        self.completed = true;
    }

    pub fn correct_chars(&self) -> usize {
        self.states.iter().filter(|s| **s == CharState::Correct).count()
    }

    pub fn incorrect_chars(&self) -> usize {
        self.states.iter().filter(|s| **s == CharState::Incorrect).count()
    }

    pub fn extra_chars(&self) -> usize {
        self.states.iter().filter(|s| **s == CharState::Extra).count()
    }

    pub fn missed_chars(&self) -> usize {
        if !self.completed {
            return 0;
        }
        if self.typed.len() < self.target.len() {
            self.target.len() - self.typed.len()
        } else {
            0
        }
    }
}

/// Delete back to previous word boundary within a Word's typed buffer.
/// Skips trailing spaces, then deletes back to the next space.
fn delete_back_word(word: &mut Word) {
    if word.typed.is_empty() {
        return;
    }

    // Skip trailing spaces/punctuation
    while !word.typed.is_empty() {
        let last = *word.typed.last().unwrap();
        if last == ' ' || last == '\t' {
            word.backspace();
        } else {
            break;
        }
    }

    // Delete back to next space or start
    while !word.typed.is_empty() {
        let last = *word.typed.last().unwrap();
        if last == ' ' || last == '\t' {
            break;
        }
        word.backspace();
    }
}

/// Per-second WPM sample for graphing
#[derive(Debug, Clone)]
pub struct WpmSample {
    pub second: u64,
    pub wpm: f64,
    pub raw_wpm: f64,
}

/// A code line for code mode: stores indent (auto-inserted) and content (user types)
#[derive(Debug, Clone)]
pub struct CodeLine {
    /// Leading whitespace (auto-inserted, user doesn't type this)
    pub indent: String,
    /// The typeable content after the indent
    pub content: Word,
}

/// The main typing test state
pub struct TypingTest {
    /// For word mode: each word is an entry
    pub words: Vec<Word>,
    pub current_word: usize,
    pub started: bool,
    pub finished: bool,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub wpm_history: Vec<WpmSample>,
    last_sample_second: u64,
    pub time_limit: Option<u64>,
    pub word_limit: Option<usize>,
    pub total_keystrokes: usize,
    pub correct_keystrokes: usize,

    /// Code mode: lines with auto-indentation
    pub code_mode: bool,
    pub code_lines: Vec<CodeLine>,
    pub current_line: usize,
}

impl TypingTest {
    pub fn new(word_strings: Vec<String>, time_limit: Option<u64>, word_limit: Option<usize>) -> Self {
        let words: Vec<Word> = word_strings.iter().map(|s| Word::new(s)).collect();
        Self {
            words,
            current_word: 0,
            started: false,
            finished: false,
            start_time: None,
            end_time: None,
            wpm_history: Vec::new(),
            last_sample_second: 0,
            time_limit,
            word_limit,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            code_mode: false,
            code_lines: Vec::new(),
            current_line: 0,
        }
    }

    /// Create a typing test for code snippets.
    /// Each line is split into indent (auto-inserted) and content (user types).
    pub fn new_code(lines: Vec<String>, time_limit: Option<u64>) -> Self {
        let code_lines: Vec<CodeLine> = lines.iter().map(|line| {
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            let indent = line[..indent_len].to_string();
            CodeLine {
                indent,
                content: Word::new(trimmed),
            }
        }).collect();

        // Also create a flat word list for stats tracking
        let words: Vec<Word> = code_lines.iter().map(|cl| cl.content.clone()).collect();

        Self {
            words,
            current_word: 0,
            started: false,
            finished: false,
            start_time: None,
            end_time: None,
            wpm_history: Vec::new(),
            last_sample_second: 0,
            time_limit,
            word_limit: None,
            total_keystrokes: 0,
            correct_keystrokes: 0,
            code_mode: true,
            code_lines,
            current_line: 0,
        }
    }

    pub fn type_char(&mut self, c: char) {
        if self.finished {
            return;
        }

        if !self.started {
            self.started = true;
            self.start_time = Some(Instant::now());
        }

        self.total_keystrokes += 1;

        if self.code_mode {
            self.type_char_code(c);
        } else {
            self.type_char_word(c);
        }

        self.maybe_sample_wpm();
        self.check_finished();
    }

    fn type_char_word(&mut self, c: char) {
        if c == ' ' {
            self.complete_word();
        } else {
            let word = &mut self.words[self.current_word];
            let pos = word.typed.len();
            word.type_char(c);
            if pos < word.target.len() && c == word.target[pos] {
                self.correct_keystrokes += 1;
            }

            // Auto-complete last word when all chars are typed
            let is_last_word = self.current_word == self.words.len() - 1;
            let word = &self.words[self.current_word];
            if is_last_word && word.typed.len() >= word.target.len() {
                self.complete_word();
            }
        }
    }

    fn type_char_code(&mut self, c: char) {
        let line = &mut self.code_lines[self.current_line];
        let word = &mut line.content;
        let pos = word.typed.len();
        word.type_char(c);
        if pos < word.target.len() && c == word.target[pos] {
            self.correct_keystrokes += 1;
        }
        // Sync with words vec for stats
        self.words[self.current_line] = line.content.clone();

        // Auto-complete last line when all chars typed
        let is_last_line = self.current_line == self.code_lines.len() - 1;
        let content = &self.code_lines[self.current_line].content;
        if is_last_line && content.typed.len() >= content.target.len() {
            self.code_lines[self.current_line].content.finalize();
            self.words[self.current_line] = self.code_lines[self.current_line].content.clone();
        }
    }

    /// In code mode, Enter advances to the next line
    pub fn enter_key(&mut self) {
        if !self.code_mode || self.finished {
            return;
        }

        if !self.started {
            self.started = true;
            self.start_time = Some(Instant::now());
        }

        self.total_keystrokes += 1;

        // Complete current line
        let line = &mut self.code_lines[self.current_line];
        line.content.finalize();
        self.words[self.current_line] = line.content.clone();

        if line.content.is_correct() {
            self.correct_keystrokes += 1;
        }

        if self.current_line + 1 < self.code_lines.len() {
            self.current_line += 1;
            self.current_word = self.current_line;
        }

        self.maybe_sample_wpm();
        self.check_finished();
    }

    pub fn backspace(&mut self) {
        if self.finished || !self.started {
            return;
        }

        if self.code_mode {
            self.backspace_code();
        } else {
            self.backspace_word();
        }
    }

    fn backspace_word(&mut self) {
        let word = &mut self.words[self.current_word];
        if word.typed.is_empty() {
            if self.current_word > 0 {
                self.current_word -= 1;
                self.words[self.current_word].completed = false;
            }
        } else {
            word.backspace();
        }
    }

    fn backspace_code(&mut self) {
        let line = &mut self.code_lines[self.current_line];
        if line.content.typed.is_empty() {
            // Go back to previous line if possible
            if self.current_line > 0 {
                self.current_line -= 1;
                self.current_word = self.current_line;
                self.code_lines[self.current_line].content.completed = false;
                self.words[self.current_line] = self.code_lines[self.current_line].content.clone();
            }
        } else {
            line.content.backspace();
            self.words[self.current_line] = line.content.clone();
        }
    }

    /// Ctrl+Backspace: delete back one word within current input.
    /// In word mode, clears the entire current word input (since each word is separate).
    /// In code mode, deletes back to the previous word boundary (space/punctuation).
    pub fn delete_word(&mut self) {
        if self.finished || !self.started {
            return;
        }
        if self.code_mode {
            let line = &mut self.code_lines[self.current_line];
            delete_back_word(&mut line.content);
            self.words[self.current_line] = line.content.clone();
        } else {
            let word = &mut self.words[self.current_word];
            word.typed.clear();
            let target_len = word.target.len();
            word.states = vec![CharState::Pending; target_len];
        }
    }

    fn complete_word(&mut self) {
        let word = &mut self.words[self.current_word];
        if word.typed.is_empty() {
            return;
        }
        word.finalize();

        if word.is_correct() {
            self.correct_keystrokes += 1;
        }

        if self.current_word + 1 < self.words.len() {
            self.current_word += 1;
        }
    }

    fn maybe_sample_wpm(&mut self) {
        if let Some(start) = self.start_time {
            let elapsed = start.elapsed().as_secs();
            if elapsed > self.last_sample_second {
                self.last_sample_second = elapsed;
                let wpm = self.calculate_wpm();
                let raw_wpm = self.calculate_raw_wpm();
                self.wpm_history.push(WpmSample {
                    second: elapsed,
                    wpm,
                    raw_wpm,
                });
            }
        }
    }

    fn check_finished(&mut self) {
        if self.finished {
            return;
        }

        if let Some(limit) = self.time_limit {
            if let Some(start) = self.start_time {
                if start.elapsed().as_secs() >= limit {
                    self.finish();
                    return;
                }
            }
        }

        if let Some(limit) = self.word_limit {
            let completed = self.words.iter().filter(|w| w.completed).count();
            if completed >= limit {
                self.finish();
                return;
            }
        }

        // Code mode: all lines completed
        if self.code_mode {
            if self.current_line >= self.code_lines.len() - 1
                && self.code_lines.last().map_or(false, |cl| cl.content.completed)
            {
                self.finish();
            }
            return;
        }

        // Word mode: all words completed
        if self.current_word >= self.words.len() - 1
            && self.words.last().map_or(false, |w| w.completed)
        {
            self.finish();
        }
    }

    pub fn check_time_limit(&mut self) {
        if let Some(limit) = self.time_limit {
            if let Some(start) = self.start_time {
                if start.elapsed().as_secs() >= limit {
                    self.finish();
                }
            }
        }
    }

    fn finish(&mut self) {
        if !self.finished {
            self.finished = true;
            self.end_time = Some(Instant::now());
            let wpm = self.calculate_wpm();
            let raw_wpm = self.calculate_raw_wpm();
            let elapsed = self.elapsed_secs();
            self.wpm_history.push(WpmSample {
                second: elapsed as u64,
                wpm,
                raw_wpm,
            });
        }
    }

    pub fn elapsed_secs(&self) -> f64 {
        match (self.start_time, self.end_time) {
            (Some(start), Some(end)) => end.duration_since(start).as_secs_f64(),
            (Some(start), None) => start.elapsed().as_secs_f64(),
            _ => 0.0,
        }
    }

    pub fn time_remaining(&self) -> Option<f64> {
        self.time_limit.map(|limit| {
            let elapsed = self.elapsed_secs();
            (limit as f64 - elapsed).max(0.0)
        })
    }

    pub fn words_completed(&self) -> usize {
        self.words.iter().filter(|w| w.completed).count()
    }

    pub fn lines_completed(&self) -> usize {
        self.code_lines.iter().filter(|cl| cl.content.completed).count()
    }

    /// WPM based on correct characters typed / 5
    pub fn calculate_wpm(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed < 0.1 {
            return 0.0;
        }
        let correct_chars: usize = self.words.iter()
            .filter(|w| w.completed)
            .map(|w| w.correct_chars() + 1) // +1 for space/enter
            .sum();
        (correct_chars as f64 / 5.0) / (elapsed / 60.0)
    }

    /// Raw WPM based on all keystrokes / 5
    pub fn calculate_raw_wpm(&self) -> f64 {
        let elapsed = self.elapsed_secs();
        if elapsed < 0.1 {
            return 0.0;
        }
        (self.total_keystrokes as f64 / 5.0) / (elapsed / 60.0)
    }

    pub fn calculate_accuracy(&self) -> f64 {
        if self.total_keystrokes == 0 {
            return 100.0;
        }
        let correct: usize = self.words.iter()
            .filter(|w| w.completed)
            .map(|w| w.correct_chars())
            .sum();
        let total: usize = self.words.iter()
            .filter(|w| w.completed)
            .map(|w| w.typed.len())
            .sum();
        if total == 0 {
            return 100.0;
        }
        (correct as f64 / total as f64) * 100.0
    }

    pub fn calculate_consistency(&self) -> f64 {
        if self.wpm_history.len() < 2 {
            return 100.0;
        }
        let wpm_values: Vec<f64> = self.wpm_history.iter().map(|s| s.wpm).collect();
        let mean = wpm_values.iter().sum::<f64>() / wpm_values.len() as f64;
        if mean < 0.1 {
            return 0.0;
        }
        let variance = wpm_values.iter().map(|v| (v - mean).powi(2)).sum::<f64>()
            / wpm_values.len() as f64;
        let std_dev = variance.sqrt();
        let cv = std_dev / mean;
        ((1.0 - cv) * 100.0).clamp(0.0, 100.0)
    }

    pub fn char_stats(&self) -> (usize, usize, usize, usize) {
        let correct: usize = self.words.iter().map(|w| w.correct_chars()).sum();
        let incorrect: usize = self.words.iter().map(|w| w.incorrect_chars()).sum();
        let extra: usize = self.words.iter().map(|w| w.extra_chars()).sum();
        let missed: usize = self.words.iter()
            .filter(|w| w.completed)
            .map(|w| w.missed_chars())
            .sum();
        (correct, incorrect, extra, missed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_correct_typing() {
        let mut word = Word::new("hello");
        for c in "hello".chars() {
            word.type_char(c);
        }
        assert!(word.is_correct());
        assert_eq!(word.correct_chars(), 5);
        assert_eq!(word.incorrect_chars(), 0);
    }

    #[test]
    fn test_word_incorrect_typing() {
        let mut word = Word::new("hello");
        for c in "hxllo".chars() {
            word.type_char(c);
        }
        assert!(!word.is_correct());
        assert_eq!(word.correct_chars(), 4);
        assert_eq!(word.incorrect_chars(), 1);
    }

    #[test]
    fn test_word_extra_chars() {
        let mut word = Word::new("hi");
        for c in "hiiii".chars() {
            word.type_char(c);
        }
        assert_eq!(word.extra_chars(), 3);
    }

    #[test]
    fn test_word_backspace() {
        let mut word = Word::new("hello");
        word.type_char('h');
        word.type_char('x');
        word.backspace();
        assert_eq!(word.typed.len(), 1);
        assert_eq!(word.states[1], CharState::Pending);
        word.type_char('e');
        assert_eq!(word.states[1], CharState::Correct);
    }

    #[test]
    fn test_typing_test_basic() {
        let words = vec!["hello".to_string(), "world".to_string()];
        let mut test = TypingTest::new(words, None, None);

        for c in "hello".chars() {
            test.type_char(c);
        }
        test.type_char(' ');
        assert_eq!(test.current_word, 1);
        assert_eq!(test.words_completed(), 1);
    }

    #[test]
    fn test_delete_word() {
        let words = vec!["hello".to_string(), "world".to_string()];
        let mut test = TypingTest::new(words, None, None);

        test.type_char('h');
        test.type_char('x');
        test.type_char('l');
        test.delete_word();

        assert_eq!(test.words[0].typed.len(), 0);
        assert!(test.words[0].states.iter().all(|s| *s == CharState::Pending));
    }

    #[test]
    fn test_delete_word_in_code_mode() {
        // In code mode, Ctrl+Backspace should delete back to previous word boundary
        let lines = vec!["hello world foo".to_string()];
        let mut test = TypingTest::new_code(lines, None);

        // Type "hello world f"
        for c in "hello world f".chars() {
            test.type_char(c);
        }
        assert_eq!(test.code_lines[0].content.typed.len(), 13);

        // Delete word: should remove "f" back to the space
        test.delete_word();
        let typed: String = test.code_lines[0].content.typed.iter().collect();
        assert_eq!(typed, "hello world ", "Should delete back to space boundary");

        // Delete again: should remove "world"
        test.delete_word();
        let typed: String = test.code_lines[0].content.typed.iter().collect();
        assert_eq!(typed, "hello ", "Should delete 'world' and trailing space");
    }

    #[test]
    fn test_code_mode_basic() {
        let lines = vec![
            "fn main() {".to_string(),
            "    println!(\"hello\");".to_string(),
            "}".to_string(),
        ];
        let mut test = TypingTest::new_code(lines, None);
        assert!(test.code_mode);
        assert_eq!(test.code_lines.len(), 3);

        // First line has no indent
        assert_eq!(test.code_lines[0].indent, "");
        assert_eq!(test.code_lines[0].content.target.iter().collect::<String>(), "fn main() {");

        // Second line has 4-space indent, user only types the content
        assert_eq!(test.code_lines[1].indent, "    ");
        assert_eq!(test.code_lines[1].content.target.iter().collect::<String>(), "println!(\"hello\");");

        // Type first line and press enter
        for c in "fn main() {".chars() {
            test.type_char(c);
        }
        test.enter_key();
        assert_eq!(test.current_line, 1);
        assert!(test.code_lines[0].content.completed);
    }

    #[test]
    fn test_auto_complete_last_word() {
        // Last word should auto-complete without needing a space
        let words = vec!["hi".to_string(), "ok".to_string()];
        let mut test = TypingTest::new(words, None, None);

        // Type "hi "
        for c in "hi".chars() { test.type_char(c); }
        test.type_char(' ');
        assert_eq!(test.current_word, 1);

        // Type "ok" - should auto-finish without space
        for c in "ok".chars() { test.type_char(c); }
        assert!(test.words[1].completed, "Last word should auto-complete");
        assert!(test.finished, "Test should be finished after typing last word");
    }

    #[test]
    fn test_auto_complete_last_code_line() {
        let lines = vec!["x = 1".to_string(), "y = 2".to_string()];
        let mut test = TypingTest::new_code(lines, None);

        // Type first line and enter
        for c in "x = 1".chars() { test.type_char(c); }
        test.enter_key();

        // Type last line - should auto-finish without enter
        for c in "y = 2".chars() { test.type_char(c); }
        assert!(test.code_lines[1].content.completed, "Last code line should auto-complete");
        assert!(test.finished, "Code test should finish after typing last line");
    }
}
