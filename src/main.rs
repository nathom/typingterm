mod app;
mod config;
mod persist;
mod theme;
mod typing;
mod ui;
mod words;

use app::{App, MenuRow, Screen};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = run_app(&mut terminal);

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    let mut app = App::new();

    loop {
        terminal.draw(|f| ui::draw(f, &app))?;

        if app.screen == Screen::Typing {
            if let Some(test) = &mut app.test {
                test.check_time_limit();
                if test.finished {
                    app.screen = Screen::Results;
                }
            }
        }

        let timeout = if app.screen == Screen::Typing {
            Duration::from_millis(50)
        } else {
            Duration::from_millis(100)
        };

        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if handle_input(&mut app, key) {
                    break;
                }
            }
        }
    }

    Ok(())
}

fn handle_input(app: &mut App, key: KeyEvent) -> bool {
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return true;
    }

    // Search mode intercepts all input
    if app.search_active {
        handle_search_input(app, key);
        return app.should_quit;
    }

    match app.screen {
        Screen::Menu => handle_menu_input(app, key),
        Screen::Typing => handle_typing_input(app, key),
        Screen::Results => handle_results_input(app, key),
    }

    app.should_quit
}

fn handle_search_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => app.cancel_search(),
        KeyCode::Enter => app.search_confirm(),
        KeyCode::Up => app.search_up(),
        KeyCode::Down => app.search_down(),
        KeyCode::Backspace => {
            if app.search_query.is_empty() {
                app.cancel_search();
            } else {
                app.search_backspace();
            }
        }
        KeyCode::Char(c) => app.search_type_char(c),
        _ => {}
    }
}

fn handle_menu_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('q') | KeyCode::Esc => {
            app.should_quit = true;
        }
        KeyCode::Char('/') => {
            // Start search on searchable rows
            if matches!(app.menu_row, MenuRow::Language | MenuRow::Theme) {
                app.start_search();
            }
        }
        KeyCode::Up | KeyCode::Char('k') => app.menu_up(),
        KeyCode::Down | KeyCode::Char('j') => app.menu_down(),
        KeyCode::Left | KeyCode::Char('h') => app.menu_left(),
        KeyCode::Right | KeyCode::Char('l') => app.menu_right(),
        KeyCode::Enter => app.start_test(),
        _ => {}
    }
}

fn handle_typing_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.back_to_menu();
        }
        KeyCode::Tab => {
            app.restart_test();
        }
        KeyCode::Enter => {
            if let Some(test) = &mut app.test {
                if test.code_mode {
                    test.enter_key();
                    if test.finished {
                        app.screen = Screen::Results;
                    }
                }
            }
        }
        KeyCode::Backspace => {
            if key.modifiers.contains(KeyModifiers::CONTROL)
                || key.modifiers.contains(KeyModifiers::ALT)
            {
                // Ctrl+Backspace or Option+Backspace: delete word
                if let Some(test) = &mut app.test {
                    test.delete_word();
                }
            } else if let Some(test) = &mut app.test {
                test.backspace();
            }
        }
        KeyCode::Char(c) => {
            if let Some(test) = &mut app.test {
                test.type_char(c);
                if test.finished {
                    app.screen = Screen::Results;
                }
            }
        }
        _ => {}
    }
}

fn handle_results_input(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Tab => {
            app.restart_test();
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            app.back_to_menu();
        }
        _ => {}
    }
}
