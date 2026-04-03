use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::app::{App, MenuRow, Screen};
use crate::config::*;
use crate::theme::Theme;
use crate::typing::CharState;

pub fn draw(f: &mut Frame, app: &App) {
    let size = f.area();

    let bg_block = Block::default().style(Style::default().bg(app.theme.bg));
    f.render_widget(bg_block, size);

    match app.screen {
        Screen::Menu => draw_menu(f, app, size),
        Screen::Typing => draw_typing(f, app, size),
        Screen::Results => draw_results(f, app, size),
    }
}

// ─────────────────────────────────────────────────────
// Menu Screen
// ─────────────────────────────────────────────────────

fn draw_menu(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // 0: title
            Constraint::Length(2),  // 1: spacer
            Constraint::Length(3),  // 2: mode
            Constraint::Length(3),  // 3: sub mode
            Constraint::Length(3),  // 4: language
            Constraint::Length(3),  // 5: theme
            Constraint::Length(3),  // 6: punctuation
            Constraint::Length(3),  // 7: numbers
            Constraint::Length(2),  // 8: spacer
            Constraint::Length(3),  // 9: hints
            Constraint::Min(0),    // 10: fill
        ])
        .horizontal_margin(4)
        .split(center_vertical(area, 31));

    // Title
    let title = Paragraph::new(Line::from(vec![
        Span::styled("typing", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled("term", Style::default().fg(theme.text)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(title, chunks[0]);

    // Mode row
    draw_menu_row(
        f, theme, chunks[2], "mode",
        &TestMode::all().iter().map(|m| m.label().to_string()).collect::<Vec<_>>(),
        TestMode::all().iter().position(|m| *m == app.config.mode).unwrap_or(0),
        app.menu_row == MenuRow::Mode,
    );

    // Sub mode row
    match app.config.mode {
        TestMode::Time => {
            draw_menu_row(
                f, theme, chunks[3], "",
                &TimeDuration::all().iter().map(|d| d.label().to_string()).collect::<Vec<_>>(),
                TimeDuration::all().iter().position(|d| *d == app.config.time_duration).unwrap_or(0),
                app.menu_row == MenuRow::SubMode,
            );
        }
        TestMode::Words => {
            draw_menu_row(
                f, theme, chunks[3], "",
                &WordCount::all().iter().map(|w| w.label().to_string()).collect::<Vec<_>>(),
                WordCount::all().iter().position(|w| *w == app.config.word_count).unwrap_or(0),
                app.menu_row == MenuRow::SubMode,
            );
        }
        _ => {
            let hint = Paragraph::new(Span::styled(
                match app.config.mode {
                    TestMode::Quote => "random quote",
                    TestMode::Zen => "zen mode - no limits",
                    _ => "",
                },
                Style::default().fg(theme.sub),
            ))
            .alignment(Alignment::Center);
            f.render_widget(hint, chunks[3]);
        }
    }

    // Language row
    let all_langs = Language::all();
    let lang_labels: Vec<String> = all_langs.iter().map(|l| l.label().to_string()).collect();
    let lang_idx = all_langs.iter().position(|l| *l == app.config.language).unwrap_or(0);
    draw_menu_row(f, theme, chunks[4], "language", &lang_labels, lang_idx, app.menu_row == MenuRow::Language);

    // Theme row
    let theme_names: Vec<String> = app.theme_catalog.names().to_vec();
    let theme_idx = app.theme_index;
    draw_menu_row(f, theme, chunks[5], "theme", &theme_names, theme_idx, app.menu_row == MenuRow::Theme);

    // Punctuation
    draw_menu_row(
        f, theme, chunks[6], "punctuation",
        &["on".to_string(), "off".to_string()],
        if app.config.punctuation { 0 } else { 1 },
        app.menu_row == MenuRow::Punctuation,
    );

    // Numbers
    draw_menu_row(
        f, theme, chunks[7], "numbers",
        &["on".to_string(), "off".to_string()],
        if app.config.numbers { 0 } else { 1 },
        app.menu_row == MenuRow::Numbers,
    );

    // Hints
    let searchable = matches!(app.menu_row, MenuRow::Language | MenuRow::Theme);
    let mut hint_spans = vec![
        Span::styled("enter", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" start  ", Style::default().fg(theme.sub)),
        Span::styled("q", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" quit", Style::default().fg(theme.sub)),
    ];
    if searchable {
        hint_spans.push(Span::styled("  ", Style::default()));
        hint_spans.push(Span::styled("/", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)));
        hint_spans.push(Span::styled(" search", Style::default().fg(theme.sub)));
    }
    let hint = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
    f.render_widget(hint, chunks[9]);

    // Search overlay
    if app.search_active {
        draw_search_overlay(f, app, area);
    }
}

fn draw_search_overlay(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;

    // Centered popup
    let popup_width = 40u16.min(area.width.saturating_sub(4));
    let popup_height = 16u16.min(area.height.saturating_sub(4));
    let popup_x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let popup_y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_width, popup_height);

    // Clear the popup area completely first (opaque background)
    let buf = f.buffer_mut();
    for y in popup_area.y..popup_area.y + popup_area.height {
        for x in popup_area.x..popup_area.x + popup_area.width {
            if x < buf.area().width && y < buf.area().height {
                buf.cell_mut((x, y)).map(|cell| {
                    cell.set_char(' ');
                    cell.set_fg(theme.text);
                    cell.set_bg(theme.bg);
                });
            }
        }
    }

    // Border and title
    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(theme.main_color))
        .style(Style::default().bg(theme.bg))
        .title(Span::styled(
            format!(" search {} ", match app.search_row {
                MenuRow::Language => "language",
                MenuRow::Theme => "theme",
                _ => "",
            }),
            Style::default().fg(theme.main_color),
        ));
    let inner = block.inner(popup_area);
    f.render_widget(block, popup_area);

    if inner.height < 2 || inner.width < 4 {
        return;
    }

    // Search input line
    let input_line = Line::from(vec![
        Span::styled("> ", Style::default().fg(theme.main_color)),
        Span::styled(&app.search_query, Style::default().fg(theme.text)),
        Span::styled("_", Style::default().fg(theme.main_color)),
    ]);
    let input_area = Rect::new(inner.x, inner.y, inner.width, 1);
    f.render_widget(Paragraph::new(input_line), input_area);

    // Results list
    let results_area = Rect::new(inner.x, inner.y + 1, inner.width, inner.height.saturating_sub(1));
    let max_visible = results_area.height as usize;

    // Scroll to keep selected visible
    let scroll = if app.search_selected >= max_visible {
        app.search_selected - max_visible + 1
    } else {
        0
    };

    let result_width = results_area.width as usize;
    let lines: Vec<Line> = app.search_results
        .iter()
        .skip(scroll)
        .take(max_visible)
        .enumerate()
        .map(|(i, name)| {
            let is_selected = i + scroll == app.search_selected;
            let display = format!(" {}", name);
            // Pad to full width so highlight fills the row
            let padded = format!("{:<width$}", display, width = result_width);
            let style = if is_selected {
                Style::default().fg(theme.bg).bg(theme.main_color)
            } else {
                Style::default().fg(theme.text).bg(theme.bg)
            };
            Line::from(Span::styled(padded, style))
        })
        .collect();

    f.render_widget(Paragraph::new(lines), results_area);
}

fn draw_menu_row(
    f: &mut Frame,
    theme: &Theme,
    area: Rect,
    label: &str,
    items: &[String],
    selected: usize,
    row_focused: bool,
) {
    let mut spans = Vec::new();

    if !label.is_empty() {
        spans.push(Span::styled(format!("{}: ", label), Style::default().fg(theme.sub)));
    }

    let (display_items, display_selected, offset) = if items.len() > 7 {
        let start = selected.saturating_sub(3).min(items.len().saturating_sub(7));
        let end = (start + 7).min(items.len());
        (items[start..end].to_vec(), selected - start, start)
    } else {
        (items.to_vec(), selected, 0)
    };

    if offset > 0 {
        spans.push(Span::styled("< ", Style::default().fg(theme.sub)));
    }

    for (i, item) in display_items.iter().enumerate() {
        if i > 0 {
            spans.push(Span::styled("  ", Style::default()));
        }

        let is_selected = i == display_selected;
        let style = if is_selected && row_focused {
            Style::default().fg(theme.bg).bg(theme.main_color).add_modifier(Modifier::BOLD)
        } else if is_selected {
            Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)
        } else if row_focused {
            Style::default().fg(theme.text)
        } else {
            Style::default().fg(theme.sub)
        };

        spans.push(Span::styled(format!(" {} ", item), style));
    }

    if offset + display_items.len() < items.len() {
        spans.push(Span::styled(" >", Style::default().fg(theme.sub)));
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

// ─────────────────────────────────────────────────────
// Typing Screen
// ─────────────────────────────────────────────────────

fn draw_typing(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    if app.test.is_none() {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(1),
            Constraint::Min(5),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .horizontal_margin(4)
        .vertical_margin(2)
        .split(area);

    draw_stats_bar(f, app, chunks[0]);

    let test = app.test.as_ref().unwrap();
    if test.code_mode {
        draw_code_text(f, app, chunks[2]);
    } else {
        draw_word_text(f, app, chunks[2]);
    }

    let enter_hint = if test.code_mode { "enter" } else { "" };
    let mut hint_spans = vec![
        Span::styled("tab", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" restart  ", Style::default().fg(theme.sub)),
        Span::styled("esc", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" menu", Style::default().fg(theme.sub)),
    ];
    if !enter_hint.is_empty() {
        hint_spans.push(Span::styled("  ", Style::default()));
        hint_spans.push(Span::styled("enter", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)));
        hint_spans.push(Span::styled(" next line", Style::default().fg(theme.sub)));
    }
    let hints = Paragraph::new(Line::from(hint_spans)).alignment(Alignment::Center);
    f.render_widget(hints, chunks[4]);
}

fn draw_stats_bar(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    let mut spans = Vec::new();

    // Code mode: show line progress
    if test.code_mode {
        let completed = test.lines_completed();
        let total = test.code_lines.len();
        spans.push(Span::styled(
            format!("{}/{}", completed, total),
            Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD),
        ));
        // Also show time if time-limited
        if let Some(remaining) = test.time_remaining() {
            spans.push(Span::styled("  ", Style::default()));
            spans.push(Span::styled(
                format!("{:.0}", remaining.ceil()),
                Style::default().fg(theme.main_color),
            ));
        }
    } else {
        match app.config.mode {
            TestMode::Time => {
                let remaining = test.time_remaining().unwrap_or(0.0);
                spans.push(Span::styled(
                    format!("{:.0}", remaining.ceil()),
                    Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD),
                ));
            }
            TestMode::Words => {
                let completed = test.words_completed();
                let total = app.config.word_count.count();
                spans.push(Span::styled(
                    format!("{}/{}", completed, total),
                    Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD),
                ));
            }
            TestMode::Quote => {
                let completed = test.words_completed();
                let total = test.words.len();
                spans.push(Span::styled(
                    format!("{}/{}", completed, total),
                    Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD),
                ));
            }
            TestMode::Zen => {
                let elapsed = test.elapsed_secs();
                spans.push(Span::styled(
                    format!("{:.0}s", elapsed),
                    Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD),
                ));
            }
        }
    }

    if test.started {
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(
            format!("{:.0} wpm", test.calculate_wpm()),
            Style::default().fg(theme.sub),
        ));
        spans.push(Span::styled("  ", Style::default()));
        spans.push(Span::styled(
            format!("{:.0}%", test.calculate_accuracy()),
            Style::default().fg(theme.sub),
        ));
    }

    let paragraph = Paragraph::new(Line::from(spans)).alignment(Alignment::Center);
    f.render_widget(paragraph, area);
}

/// Render word-mode typing text
fn draw_word_text(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    let max_width = area.width as usize;
    let mut lines: Vec<Line> = Vec::new();
    let mut current_line_spans: Vec<Span> = Vec::new();
    let mut current_line_width: usize = 0;

    let mut cursor_line: u16 = 0;
    let mut cursor_col: u16 = 0;
    let mut found_cursor = false;

    for (word_idx, word) in test.words.iter().enumerate() {
        let is_current = word_idx == test.current_word;
        let word_display_len = word.target.len().max(word.typed.len());
        let needed = if current_line_width > 0 { word_display_len + 1 } else { word_display_len };

        if current_line_width > 0 && current_line_width + needed > max_width {
            lines.push(Line::from(current_line_spans.clone()));
            current_line_spans.clear();
            current_line_width = 0;
            if lines.len() >= area.height as usize {
                break;
            }
        }

        if current_line_width > 0 {
            current_line_spans.push(Span::styled(" ", Style::default().fg(theme.text_dim)));
            current_line_width += 1;
        }

        if is_current && !found_cursor {
            cursor_line = lines.len() as u16;
            cursor_col = current_line_width as u16 + word.cursor_pos() as u16;
            found_cursor = true;
        }

        render_word_spans(&mut current_line_spans, &mut current_line_width, word, theme);
    }

    if !current_line_spans.is_empty() {
        lines.push(Line::from(current_line_spans));
    }

    let visible_height = area.height as usize;
    let scroll_offset = if cursor_line as usize >= visible_height {
        cursor_line as usize - visible_height + 1
    } else {
        0
    };

    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll_offset).take(visible_height).collect();
    let paragraph = Paragraph::new(visible_lines);
    f.render_widget(paragraph, area);

    draw_cursor(f, app, area, cursor_line, cursor_col, scroll_offset, found_cursor, false);
}

/// Render code-mode typing text (line-by-line with indentation)
fn draw_code_text(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    let mut lines: Vec<Line> = Vec::new();
    let mut cursor_line: u16 = 0;
    let mut cursor_col: u16 = 0;
    let mut found_cursor = false;

    for (line_idx, code_line) in test.code_lines.iter().enumerate() {
        let is_current = line_idx == test.current_line;
        let word = &code_line.content;
        let indent = &code_line.indent;

        let mut spans: Vec<Span> = Vec::new();
        let mut line_width: usize = 0;

        // Render indent (dimmed, auto-inserted)
        if !indent.is_empty() {
            let indent_display = indent.replace('\t', "    ");
            spans.push(Span::styled(
                indent_display.clone(),
                Style::default().fg(theme.sub),
            ));
            line_width += indent_display.len();
        }

        // Track cursor
        if is_current && !found_cursor {
            cursor_line = lines.len() as u16;
            cursor_col = line_width as u16 + word.cursor_pos() as u16;
            found_cursor = true;
        }

        // Render the typeable content
        render_word_spans(&mut spans, &mut line_width, word, theme);

        // Show a dimmed enter symbol at end of line (except last)
        if line_idx < test.code_lines.len() - 1 {
            spans.push(Span::styled(" \u{21b5}", Style::default().fg(theme.sub)));
        }

        lines.push(Line::from(spans));
    }

    let visible_height = area.height as usize;
    let scroll_offset = if cursor_line as usize >= visible_height {
        cursor_line as usize - visible_height + 1
    } else {
        0
    };

    let visible_lines: Vec<Line> = lines.into_iter().skip(scroll_offset).take(visible_height).collect();
    let paragraph = Paragraph::new(visible_lines);
    f.render_widget(paragraph, area);

    draw_cursor(f, app, area, cursor_line, cursor_col, scroll_offset, found_cursor, true);
}

/// Render characters of a word into spans
fn render_word_spans<'a>(
    spans: &mut Vec<Span<'a>>,
    line_width: &mut usize,
    word: &crate::typing::Word,
    theme: &Theme,
) {
    for (char_idx, target_char) in word.target.iter().enumerate() {
        let (ch, style) = if char_idx < word.typed.len() {
            let state = word.states[char_idx];
            let typed_char = word.typed[char_idx];
            match state {
                CharState::Correct => (*target_char, Style::default().fg(theme.correct)),
                CharState::Incorrect => (typed_char, Style::default().fg(theme.incorrect).bg(theme.error_bg)),
                _ => (*target_char, Style::default().fg(theme.text_dim)),
            }
        } else {
            (*target_char, Style::default().fg(theme.text_dim))
        };

        spans.push(Span::styled(ch.to_string(), style));
        *line_width += 1;
    }

    // Extra characters
    if word.typed.len() > word.target.len() {
        for i in word.target.len()..word.typed.len() {
            spans.push(Span::styled(
                word.typed[i].to_string(),
                Style::default().fg(theme.extra).bg(theme.error_bg),
            ));
            *line_width += 1;
        }
    }
}

/// Draw the cursor caret on the buffer
fn draw_cursor(
    f: &mut Frame,
    app: &App,
    area: Rect,
    cursor_line: u16,
    cursor_col: u16,
    scroll_offset: usize,
    found_cursor: bool,
    code_mode: bool,
) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    if !found_cursor || test.finished {
        return;
    }

    let cursor_y = area.y + cursor_line - scroll_offset as u16;
    let cursor_x = area.x + cursor_col;

    if cursor_y >= area.y + area.height || cursor_x >= area.x + area.width {
        return;
    }

    let cursor_char = if code_mode {
        let cl = &test.code_lines[test.current_line];
        let pos = cl.content.cursor_pos();
        if pos < cl.content.target.len() {
            cl.content.target[pos]
        } else {
            ' '
        }
    } else if test.current_word < test.words.len() {
        let word = &test.words[test.current_word];
        let pos = word.cursor_pos();
        if pos < word.target.len() {
            word.target[pos]
        } else {
            ' '
        }
    } else {
        ' '
    };

    let cursor_span = Span::styled(
        cursor_char.to_string(),
        Style::default().fg(theme.bg).bg(theme.caret),
    );
    let buf = f.buffer_mut();
    if cursor_x < buf.area().width && cursor_y < buf.area().height {
        buf.set_span(cursor_x, cursor_y, &cursor_span, 1);
    }
}

// ─────────────────────────────────────────────────────
// Results Screen
// ─────────────────────────────────────────────────────

fn draw_results(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    if app.test.is_none() {
        return;
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(2),
            Constraint::Length(6),
            Constraint::Length(1),
            Constraint::Min(8),
            Constraint::Length(1),
            Constraint::Length(4),
            Constraint::Length(2),
            Constraint::Length(2),
        ])
        .horizontal_margin(4)
        .vertical_margin(1)
        .split(area);

    draw_main_stats(f, app, chunks[1]);
    draw_wpm_graph(f, app, chunks[3]);
    draw_char_stats(f, app, chunks[5]);

    let hints = Paragraph::new(Line::from(vec![
        Span::styled("tab", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" restart  ", Style::default().fg(theme.sub)),
        Span::styled("esc", Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD)),
        Span::styled(" menu", Style::default().fg(theme.sub)),
    ]))
    .alignment(Alignment::Center);
    f.render_widget(hints, chunks[7]);
}

fn draw_main_stats(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    let stats_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let wpm = test.calculate_wpm();
    let raw = test.calculate_raw_wpm();
    let acc = test.calculate_accuracy();
    let consistency = test.calculate_consistency();

    let stats = [
        ("wpm", format!("{:.0}", wpm)),
        ("raw", format!("{:.0}", raw)),
        ("accuracy", format!("{:.1}%", acc)),
        ("consistency", format!("{:.0}%", consistency)),
    ];

    for (i, (label, value)) in stats.iter().enumerate() {
        let lines = vec![
            Line::from(Span::styled(value.clone(), Style::default().fg(theme.main_color).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(label.to_string(), Style::default().fg(theme.sub))),
        ];
        let p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(p, stats_chunks[i]);
    }
}

fn draw_wpm_graph(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();

    if test.wpm_history.is_empty() || area.height < 3 || area.width < 10 {
        return;
    }

    let block = Block::default()
        .borders(Borders::LEFT | Borders::BOTTOM)
        .border_style(Style::default().fg(theme.sub))
        .title(Span::styled("wpm", Style::default().fg(theme.sub)));
    let inner = block.inner(area);
    f.render_widget(block, area);

    if inner.height == 0 || inner.width == 0 {
        return;
    }

    let samples = &test.wpm_history;
    let max_wpm = samples
        .iter()
        .map(|s| s.wpm.max(s.raw_wpm))
        .fold(0.0f64, f64::max)
        .max(10.0);
    // Round up to a nice number
    let max_wpm = ((max_wpm / 10.0).ceil() * 10.0).max(20.0);

    let total_seconds = samples.last().map(|s| s.second).unwrap_or(1).max(1) as f64;
    let buf = f.buffer_mut();
    let w = inner.width as f64;
    let h = inner.height as f64;

    // Helper: map a (second, wpm) point to (column, row) in the inner area
    let map_point = |sec: f64, wpm: f64| -> (u16, u16) {
        let x = ((sec / total_seconds) * (w - 1.0)).round().clamp(0.0, w - 1.0) as u16;
        let y_frac = (wpm / max_wpm).clamp(0.0, 1.0);
        let y = ((1.0 - y_frac) * (h - 1.0)).round().clamp(0.0, h - 1.0) as u16;
        (inner.x + x, inner.y + y)
    };

    // Draw raw WPM line (dim dots)
    draw_line_graph(buf, &samples.iter().map(|s| (s.second as f64, s.raw_wpm)).collect::<Vec<_>>(), &map_point, theme.sub, '·');

    // Draw WPM line (bright, connected)
    draw_line_graph(buf, &samples.iter().map(|s| (s.second as f64, s.wpm)).collect::<Vec<_>>(), &map_point, theme.main_color, '•');

    // Y-axis labels
    let top_label = format!("{:.0}", max_wpm);
    for (i, ch) in top_label.chars().enumerate() {
        let x = area.x + i as u16;
        if x < area.x + area.width && inner.y < buf.area().height {
            buf.cell_mut((x, inner.y)).map(|cell| {
                cell.set_char(ch);
                cell.set_fg(theme.sub);
            });
        }
    }

    // Mid label
    let mid_label = format!("{:.0}", max_wpm / 2.0);
    let mid_y = inner.y + inner.height / 2;
    for (i, ch) in mid_label.chars().enumerate() {
        let x = area.x + i as u16;
        if x < area.x + area.width && mid_y < buf.area().height {
            buf.cell_mut((x, mid_y)).map(|cell| {
                cell.set_char(ch);
                cell.set_fg(theme.sub);
            });
        }
    }
}

/// Draw a line graph by plotting points and connecting consecutive ones with interpolation
fn draw_line_graph(
    buf: &mut ratatui::buffer::Buffer,
    points: &[(f64, f64)],
    map_point: &dyn Fn(f64, f64) -> (u16, u16),
    color: Color,
    point_char: char,
) {
    if points.is_empty() {
        return;
    }

    let buf_w = buf.area().width;
    let buf_h = buf.area().height;

    for i in 0..points.len() {
        let (x1, y1) = map_point(points[i].0, points[i].1);

        // Draw the point itself
        if x1 < buf_w && y1 < buf_h {
            buf.cell_mut((x1, y1)).map(|cell| {
                cell.set_char(point_char);
                cell.set_fg(color);
            });
        }

        // Connect to next point with line characters
        if i + 1 < points.len() {
            let (x2, y2) = map_point(points[i + 1].0, points[i + 1].1);

            // Interpolate between the two points
            let dx = (x2 as i32) - (x1 as i32);
            let dy = (y2 as i32) - (y1 as i32);
            let steps = dx.abs().max(dy.abs()).max(1);

            for step in 1..steps {
                let t = step as f64 / steps as f64;
                let ix = (x1 as f64 + t * dx as f64).round() as u16;
                let iy = (y1 as f64 + t * dy as f64).round() as u16;

                if ix < buf_w && iy < buf_h {
                    // Choose line character based on direction
                    let ch = if dy.abs() > dx.abs() * 2 {
                        '│' // steep vertical
                    } else if dx.abs() > dy.abs() * 2 {
                        '─' // mostly horizontal
                    } else if (dy > 0) == (dx > 0) {
                        '╲' // going down-right or up-left
                    } else {
                        '╱' // going up-right or down-left
                    };

                    buf.cell_mut((ix, iy)).map(|cell| {
                        // Don't overwrite actual data points
                        if cell.symbol() != &point_char.to_string() {
                            cell.set_char(ch);
                            cell.set_fg(color);
                        }
                    });
                }
            }
        }
    }
}

fn draw_char_stats(f: &mut Frame, app: &App, area: Rect) {
    let theme = &app.theme;
    let test = app.test.as_ref().unwrap();
    let (correct, incorrect, extra, missed) = test.char_stats();

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ])
        .split(area);

    let stats = [
        ("correct", correct, Color::Rgb(120, 180, 80)),
        ("incorrect", incorrect, Color::Rgb(202, 71, 71)),
        ("extra", extra, Color::Rgb(180, 120, 50)),
        ("missed", missed, Color::Rgb(150, 150, 150)),
    ];

    for (i, (label, count, color)) in stats.iter().enumerate() {
        let lines = vec![
            Line::from(Span::styled(count.to_string(), Style::default().fg(*color).add_modifier(Modifier::BOLD))),
            Line::from(Span::styled(label.to_string(), Style::default().fg(theme.sub))),
        ];
        let p = Paragraph::new(lines).alignment(Alignment::Center);
        f.render_widget(p, chunks[i]);
    }
}

// ─────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────

fn center_vertical(area: Rect, height: u16) -> Rect {
    let height = height.min(area.height);
    let y = area.y + (area.height.saturating_sub(height)) / 2;
    Rect::new(area.x, y, area.width, height)
}
