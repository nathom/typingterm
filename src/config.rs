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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Language {
    English,
    English1k,
    English5k,
    English10k,
    Spanish,
    French,
    German,
    Portuguese,
    Italian,
    Dutch,
    Swedish,
    CodeRust,
    CodePython,
    CodeJavascript,
    CodeTypescript,
    CodeGo,
    CodeC,
    CodeCpp,
    CodeJava,
}

impl Language {
    pub fn label(&self) -> &str {
        match self {
            Language::English => "english",
            Language::English1k => "english 1k",
            Language::English5k => "english 5k",
            Language::English10k => "english 10k",
            Language::Spanish => "spanish",
            Language::French => "french",
            Language::German => "german",
            Language::Portuguese => "portuguese",
            Language::Italian => "italian",
            Language::Dutch => "dutch",
            Language::Swedish => "swedish",
            Language::CodeRust => "code rust",
            Language::CodePython => "code python",
            Language::CodeJavascript => "code javascript",
            Language::CodeTypescript => "code typescript",
            Language::CodeGo => "code go",
            Language::CodeC => "code c",
            Language::CodeCpp => "code c++",
            Language::CodeJava => "code java",
        }
    }

    pub fn natural_languages() -> &'static [Language] {
        &[
            Language::English,
            Language::English1k,
            Language::English5k,
            Language::English10k,
            Language::Spanish,
            Language::French,
            Language::German,
            Language::Portuguese,
            Language::Italian,
            Language::Dutch,
            Language::Swedish,
        ]
    }

    pub fn code_languages() -> &'static [Language] {
        &[
            Language::CodeRust,
            Language::CodePython,
            Language::CodeJavascript,
            Language::CodeTypescript,
            Language::CodeGo,
            Language::CodeC,
            Language::CodeCpp,
            Language::CodeJava,
        ]
    }

    pub fn all() -> Vec<Language> {
        let mut all = Vec::new();
        all.extend_from_slice(Self::natural_languages());
        all.extend_from_slice(Self::code_languages());
        all
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestConfig {
    pub mode: TestMode,
    pub time_duration: TimeDuration,
    pub word_count: WordCount,
    pub language: Language,
    pub punctuation: bool,
    pub numbers: bool,
}

impl Default for TestConfig {
    fn default() -> Self {
        Self {
            mode: TestMode::Time,
            time_duration: TimeDuration::Thirty,
            word_count: WordCount::TwentyFive,
            language: Language::English,
            punctuation: false,
            numbers: false,
        }
    }
}
