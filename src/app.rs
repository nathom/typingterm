use crate::config::*;
use crate::persist::PersistentConfig;
use crate::theme::{Theme, ThemeCatalog};
use crate::typing::TypingTest;
use crate::words;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Menu,
    Typing,
    Results,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuRow {
    Mode,
    SubMode,
    Language,
    Theme,
    Punctuation,
    Numbers,
}

impl MenuRow {
    pub fn all() -> &'static [MenuRow] {
        &[
            MenuRow::Mode,
            MenuRow::SubMode,
            MenuRow::Language,
            MenuRow::Theme,
            MenuRow::Punctuation,
            MenuRow::Numbers,
        ]
    }

    pub fn index(&self) -> usize {
        match self {
            MenuRow::Mode => 0,
            MenuRow::SubMode => 1,
            MenuRow::Language => 2,
            MenuRow::Theme => 3,
            MenuRow::Punctuation => 4,
            MenuRow::Numbers => 5,
        }
    }

    pub fn from_index(i: usize) -> Self {
        match i {
            0 => MenuRow::Mode,
            1 => MenuRow::SubMode,
            2 => MenuRow::Language,
            3 => MenuRow::Theme,
            4 => MenuRow::Punctuation,
            5 => MenuRow::Numbers,
            _ => MenuRow::Mode,
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub config: TestConfig,
    pub test: Option<TypingTest>,
    pub theme: Theme,
    pub theme_catalog: ThemeCatalog,
    pub should_quit: bool,

    // Menu state
    pub menu_row: MenuRow,
    pub menu_col: usize,
    pub language_names: Vec<&'static str>,
    pub language_index: usize,
    pub theme_index: usize,

    // Search mode
    pub search_active: bool,
    pub search_query: String,
    pub search_results: Vec<String>,
    pub search_selected: usize,
    pub search_row: MenuRow,
}

impl App {
    pub fn new() -> Self {
        let catalog = ThemeCatalog::load();
        let saved = PersistentConfig::load();

        let theme_name = saved.theme.as_deref().unwrap_or("serika_dark");
        let theme_name = if catalog.names().iter().any(|n| n == theme_name) {
            theme_name
        } else {
            catalog.names().first().map(|s| s.as_str()).unwrap_or("serika_dark")
        };
        let theme = catalog.get(theme_name);
        let theme_index = catalog.names().iter().position(|n| n == theme_name).unwrap_or(0);

        let mut config = TestConfig::default();
        if let Some(lang_str) = &saved.language {
            config.language = lang_str.clone();
        }
        if let Some(p) = saved.punctuation {
            config.punctuation = p;
        }
        if let Some(n) = saved.numbers {
            config.numbers = n;
        }

        let language_names = words::all_language_names();
        let language_index = language_names
            .iter()
            .position(|n| *n == config.language.as_str())
            .unwrap_or(0);

        Self {
            screen: Screen::Menu,
            config,
            test: None,
            theme,
            theme_catalog: catalog,
            should_quit: false,
            menu_row: MenuRow::Mode,
            menu_col: 0,
            language_names,
            language_index,
            theme_index,
            search_active: false,
            search_query: String::new(),
            search_results: Vec::new(),
            search_selected: 0,
            search_row: MenuRow::Language,
        }
    }

    pub fn save_config(&self) {
        let config = PersistentConfig {
            theme: Some(self.theme.name.clone()),
            language: Some(self.config.language.clone()),
            mode: Some(self.config.mode.label().to_string()),
            time_duration: Some(self.config.time_duration.seconds()),
            word_count: Some(self.config.word_count.count()),
            punctuation: Some(self.config.punctuation),
            numbers: Some(self.config.numbers),
        };
        config.save();
    }

    // --- Search mode ---

    pub fn start_search(&mut self) {
        self.search_active = true;
        self.search_query.clear();
        self.search_selected = 0;
        self.search_row = self.menu_row;
        self.update_search_results();
    }

    pub fn cancel_search(&mut self) {
        self.search_active = false;
        self.search_query.clear();
        self.search_results.clear();
    }

    pub fn search_type_char(&mut self, c: char) {
        self.search_query.push(c);
        self.search_selected = 0;
        self.update_search_results();
    }

    pub fn search_backspace(&mut self) {
        self.search_query.pop();
        self.search_selected = 0;
        self.update_search_results();
    }

    pub fn search_up(&mut self) {
        if self.search_selected > 0 {
            self.search_selected -= 1;
        }
    }

    pub fn search_down(&mut self) {
        if self.search_selected + 1 < self.search_results.len() {
            self.search_selected += 1;
        }
    }

    pub fn search_confirm(&mut self) {
        if let Some(selected) = self.search_results.get(self.search_selected).cloned() {
            match self.search_row {
                MenuRow::Language => {
                    if let Some(idx) = self.language_names.iter().position(|n| *n == selected.as_str()) {
                        self.config.language = selected;
                        self.language_index = idx;
                    }
                }
                MenuRow::Theme => {
                    self.theme = self.theme_catalog.get(&selected);
                    self.theme_index = self.theme_catalog
                        .names()
                        .iter()
                        .position(|n| n == &selected)
                        .unwrap_or(0);
                }
                _ => {}
            }
        }
        self.cancel_search();
        self.save_config();
    }

    fn update_search_results(&mut self) {
        let q = self.search_query.to_lowercase();
        match self.search_row {
            MenuRow::Language => {
                self.search_results = self.language_names
                    .iter()
                    .filter(|n| q.is_empty() || n.to_lowercase().contains(&q))
                    .map(|n| n.to_string())
                    .collect();
            }
            MenuRow::Theme => {
                if q.is_empty() {
                    self.search_results = self.theme_catalog.names().to_vec();
                } else {
                    self.search_results = self
                        .theme_catalog
                        .search(&q)
                        .into_iter()
                        .map(|s| s.to_string())
                        .collect();
                }
            }
            _ => {
                self.search_results.clear();
            }
        }
    }

    // --- Test management ---

    pub fn start_test(&mut self) {
        if words::is_code_language(&self.config.language) {
            let lines = words::get_code_snippet(&self.config.language);
            let time_limit = match self.config.mode {
                TestMode::Time => Some(self.config.time_duration.seconds()),
                _ => None,
            };
            self.test = Some(TypingTest::new_code(lines, time_limit));
            self.screen = Screen::Typing;
            return;
        }

        let word_count = match self.config.mode {
            TestMode::Time => 200,
            TestMode::Words => self.config.word_count.count(),
            TestMode::Quote => 0,
            TestMode::Zen => 500,
        };

        let (time_limit, word_limit) = match self.config.mode {
            TestMode::Time => (Some(self.config.time_duration.seconds()), None),
            TestMode::Words => (None, Some(self.config.word_count.count())),
            TestMode::Quote => (None, None),
            TestMode::Zen => (None, None),
        };

        let word_strings = if self.config.mode == TestMode::Quote {
            let (_source, text) = words::get_random_quote();
            text.split_whitespace().map(|s| s.to_string()).collect()
        } else {
            words::generate_words(
                &self.config.language,
                word_count,
                self.config.punctuation,
                self.config.numbers,
            )
        };

        self.test = Some(TypingTest::new(word_strings, time_limit, word_limit));
        self.screen = Screen::Typing;
    }

    pub fn restart_test(&mut self) {
        self.start_test();
    }

    pub fn back_to_menu(&mut self) {
        self.test = None;
        self.screen = Screen::Menu;
    }

    // --- Menu navigation ---

    pub fn menu_up(&mut self) {
        let idx = self.menu_row.index();
        if idx > 0 {
            self.menu_row = MenuRow::from_index(idx - 1);
            self.sync_menu_col();
        }
    }

    pub fn menu_down(&mut self) {
        let idx = self.menu_row.index();
        let rows = MenuRow::all();
        if idx + 1 < rows.len() {
            self.menu_row = MenuRow::from_index(idx + 1);
            self.sync_menu_col();
        }
    }

    pub fn menu_left(&mut self) {
        if self.menu_col > 0 {
            self.menu_col -= 1;
            self.apply_menu_selection();
        }
    }

    pub fn menu_right(&mut self) {
        let max = self.menu_row_len();
        if self.menu_col + 1 < max {
            self.menu_col += 1;
            self.apply_menu_selection();
        }
    }

    fn menu_row_len(&self) -> usize {
        match self.menu_row {
            MenuRow::Mode => TestMode::all().len(),
            MenuRow::SubMode => match self.config.mode {
                TestMode::Time => TimeDuration::all().len(),
                TestMode::Words => WordCount::all().len(),
                _ => 0,
            },
            MenuRow::Language => self.language_names.len(),
            MenuRow::Theme => self.theme_catalog.count(),
            MenuRow::Punctuation => 2,
            MenuRow::Numbers => 2,
        }
    }

    fn sync_menu_col(&mut self) {
        self.menu_col = match self.menu_row {
            MenuRow::Mode => TestMode::all()
                .iter()
                .position(|m| *m == self.config.mode)
                .unwrap_or(0),
            MenuRow::SubMode => match self.config.mode {
                TestMode::Time => TimeDuration::all()
                    .iter()
                    .position(|d| *d == self.config.time_duration)
                    .unwrap_or(0),
                TestMode::Words => WordCount::all()
                    .iter()
                    .position(|w| *w == self.config.word_count)
                    .unwrap_or(0),
                _ => 0,
            },
            MenuRow::Language => self.language_index,
            MenuRow::Theme => self.theme_index,
            MenuRow::Punctuation => {
                if self.config.punctuation { 0 } else { 1 }
            }
            MenuRow::Numbers => {
                if self.config.numbers { 0 } else { 1 }
            }
        };
    }

    fn apply_menu_selection(&mut self) {
        match self.menu_row {
            MenuRow::Mode => {
                if let Some(mode) = TestMode::all().get(self.menu_col) {
                    self.config.mode = *mode;
                }
            }
            MenuRow::SubMode => match self.config.mode {
                TestMode::Time => {
                    if let Some(d) = TimeDuration::all().get(self.menu_col) {
                        self.config.time_duration = *d;
                    }
                }
                TestMode::Words => {
                    if let Some(w) = WordCount::all().get(self.menu_col) {
                        self.config.word_count = *w;
                    }
                }
                _ => {}
            },
            MenuRow::Language => {
                if let Some(name) = self.language_names.get(self.menu_col) {
                    self.config.language = name.to_string();
                    self.language_index = self.menu_col;
                }
            }
            MenuRow::Theme => {
                if let Some(name) = self.theme_catalog.names().get(self.menu_col) {
                    self.theme = self.theme_catalog.get(name);
                    self.theme_index = self.menu_col;
                }
            }
            MenuRow::Punctuation => {
                self.config.punctuation = self.menu_col == 0;
            }
            MenuRow::Numbers => {
                self.config.numbers = self.menu_col == 0;
            }
        }
        self.save_config();
    }
}
