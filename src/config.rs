use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TestMode {
    Time,
    Words,
    Quote,
    Zen,
}

impl TestMode {
    pub fn label(&self) -> &str {
        match self {
            TestMode::Time => "time",
            TestMode::Words => "words",
            TestMode::Quote => "quote",
            TestMode::Zen => "zen",
        }
    }

    pub fn all() -> &'static [TestMode] {
        &[TestMode::Time, TestMode::Words, TestMode::Quote, TestMode::Zen]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeDuration {
    Fifteen,
    Thirty,
    Sixty,
    OneHundredTwenty,
}

impl TimeDuration {
    pub fn seconds(&self) -> u64 {
        match self {
            TimeDuration::Fifteen => 15,
            TimeDuration::Thirty => 30,
            TimeDuration::Sixty => 60,
            TimeDuration::OneHundredTwenty => 120,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            TimeDuration::Fifteen => "15",
            TimeDuration::Thirty => "30",
            TimeDuration::Sixty => "60",
            TimeDuration::OneHundredTwenty => "120",
        }
    }

    pub fn all() -> &'static [TimeDuration] {
        &[
            TimeDuration::Fifteen,
            TimeDuration::Thirty,
            TimeDuration::Sixty,
            TimeDuration::OneHundredTwenty,
        ]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WordCount {
    Ten,
    TwentyFive,
    Fifty,
    Hundred,
}

impl WordCount {
    pub fn count(&self) -> usize {
        match self {
            WordCount::Ten => 10,
            WordCount::TwentyFive => 25,
            WordCount::Fifty => 50,
            WordCount::Hundred => 100,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            WordCount::Ten => "10",
            WordCount::TwentyFive => "25",
            WordCount::Fifty => "50",
            WordCount::Hundred => "100",
        }
    }

    pub fn all() -> &'static [WordCount] {
        &[
            WordCount::Ten,
            WordCount::TwentyFive,
            WordCount::Fifty,
            WordCount::Hundred,
        ]
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub mode: TestMode,
    pub time_duration: TimeDuration,
    pub word_count: WordCount,
    pub language: String,
    pub punctuation: bool,
    pub numbers: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            mode: TestMode::Time,
            time_duration: TimeDuration::Thirty,
            word_count: WordCount::TwentyFive,
            language: "english".to_string(),
            punctuation: false,
            numbers: false,
        }
    }
}
