//! TUI module for managing terminal interface with ratatui

use crate::log_entry::LogEntry;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, Borders, List, ListItem, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
    },
};
use std::io::{self, Stdout, stdout};

/// `CrosstermTerminal` is an alias for the `CrossTerm` backend.
pub type CrosstermTerminal = Terminal<CrosstermBackend<Stdout>>;

/// `Tui` manages the terminal user interface using ratatui
pub struct Tui {
    /// `terminal` is the terminal instance doing all the work.
    terminal: CrosstermTerminal,
    /// `log_entries` are the log entries that will be displayed to the screen.
    log_entries: Vec<LogEntry>,
    /// `scroll_offset` is the amount of offset that the screen has to scroll to show the correct
    /// log entries.
    scroll_offset: usize,
    /// `selected_index` is the current log entry that's highlighted in the UI.
    selected_index: Option<usize>,
    /// `auto_scroll` keeps the window at the bottom of the log file when true.
    auto_scroll: bool, // Track if we should auto-scroll to bottom
}

impl Tui {
    /// Create a new TUI instance
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(stdout());
        let terminal = Terminal::new(backend)?;

        Ok(Self {
            terminal,
            log_entries: Vec::new(),
            scroll_offset: 0,
            selected_index: None,
            auto_scroll: true,
        })
    }

    /// Start the TUI by enabling raw mode and entering alternate screen
    pub fn start(&mut self) -> io::Result<()> {
        enable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            EnterAlternateScreen,
            EnableMouseCapture
        )?;
        self.terminal.clear()?;
        Ok(())
    }

    /// End the TUI by disabling raw mode and leaving alternate screen
    pub fn end(&mut self) -> io::Result<()> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }

    /// Set all log entries (replaces existing entries)
    pub fn set_log_entries(&mut self, entries: Vec<LogEntry>) {
        self.log_entries = entries;
        // Don't auto-scroll on initial load, let user see from the beginning
        self.auto_scroll = false;
        self.scroll_offset = 0;
    }

    /// Add new log entries (for when the source only provides new entries)
    pub fn append_new_log_entries(&mut self, new_entries: Vec<LogEntry>) {
        if new_entries.is_empty() {
            return;
        }

        // Check if we're at the bottom AND auto-scroll is enabled
        let should_auto_scroll = self.auto_scroll && self.is_at_bottom();

        // Add the new entries
        self.log_entries.extend(new_entries);

        // Only auto-scroll if both conditions are met:
        // 1. User was already at the bottom
        // 2. Auto-scroll mode is enabled (not paused)
        if should_auto_scroll {
            self.scroll_to_show_latest();
        }
    }

    /// Scroll just enough to show the latest entries (minimal scrolling)
    fn scroll_to_show_latest(&mut self) {
        let terminal_height = self.terminal.size().unwrap_or_default().height as usize;
        let content_height = terminal_height.saturating_sub(4); // Account for borders and title

        if self.log_entries.len() > content_height {
            // Calculate the scroll offset to show the last `content_height` entries
            // This ensures we see a full screen with the newest entries at the bottom
            let new_scroll_offset = self.log_entries.len().saturating_sub(content_height);
            self.scroll_offset = new_scroll_offset;
        } else {
            // If all entries fit on screen, no need to scroll
            self.scroll_offset = 0;
        }
    }

    /// Check if the user is currently viewing the bottom of the log
    fn is_at_bottom(&self) -> bool {
        if self.log_entries.is_empty() {
            return true;
        }

        let terminal_height = self.terminal.size().unwrap_or_default().height as usize;
        let content_height = terminal_height.saturating_sub(4); // Account for borders and title

        if self.log_entries.len() <= content_height {
            return true; // All entries fit on screen
        }

        let max_scroll = self.log_entries.len().saturating_sub(content_height);
        self.scroll_offset >= max_scroll
    }

    /// Clear all log entries
    pub fn clear_log_entries(&mut self) {
        self.log_entries.clear();
        self.scroll_offset = 0;
        self.selected_index = None;
        self.auto_scroll = true; // Re-enable auto-scroll after clearing
    }

    /// Scroll to show the latest entries (keeps screen full)
    pub fn scroll_to_bottom(&mut self) {
        if !self.log_entries.is_empty() {
            let terminal_height = self.terminal.size().unwrap_or_default().height as usize;
            let content_height = terminal_height.saturating_sub(4); // Account for borders and title

            if self.log_entries.len() > content_height {
                // Set scroll offset so the last entry is at the bottom of the visible area
                self.scroll_offset = self.log_entries.len().saturating_sub(content_height);
            } else {
                self.scroll_offset = 0;
            }
        }
    }

    /// Handle keyboard input and return whether to continue running
    pub fn handle_input(&mut self) -> io::Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Char('q') | KeyCode::Esc => return Ok(false),
                        KeyCode::Up | KeyCode::Char('k') => {
                            if self.scroll_offset > 0 {
                                self.scroll_offset -= 1;
                                // Disable auto-scroll when user manually scrolls up
                                self.auto_scroll = false;
                            }
                        }
                        KeyCode::Down | KeyCode::Char('j') => {
                            let terminal_height = self.terminal.size()?.height as usize;
                            let content_height = terminal_height.saturating_sub(4);
                            let max_scroll = self.log_entries.len().saturating_sub(content_height);

                            if self.scroll_offset < max_scroll {
                                self.scroll_offset += 1;
                                // Check if we've scrolled back to the bottom
                                if self.scroll_offset >= max_scroll {
                                    self.auto_scroll = true;
                                }
                            }
                        }
                        KeyCode::PageUp => {
                            let page_size = 10;
                            self.scroll_offset = self.scroll_offset.saturating_sub(page_size);
                            self.auto_scroll = false;
                        }
                        KeyCode::PageDown => {
                            let terminal_height = self.terminal.size()?.height as usize;
                            let content_height = terminal_height.saturating_sub(4);
                            let max_scroll = self.log_entries.len().saturating_sub(content_height);
                            let page_size = 10;

                            self.scroll_offset = (self.scroll_offset + page_size).min(max_scroll);
                            // Check if we've scrolled back to the bottom
                            if self.scroll_offset >= max_scroll {
                                self.auto_scroll = true;
                            }
                        }
                        KeyCode::Home => {
                            self.scroll_offset = 0;
                            self.auto_scroll = false;
                        }
                        KeyCode::End => {
                            self.scroll_to_bottom();
                            self.auto_scroll = true;
                        }
                        KeyCode::Char('c') => {
                            self.clear_log_entries();
                        }
                        KeyCode::Char('f') => {
                            // Toggle auto-follow mode
                            self.auto_scroll = !self.auto_scroll;
                            if self.auto_scroll {
                                self.scroll_to_bottom();
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(true)
    }

    /// Render the TUI
    pub fn render(&mut self) -> io::Result<()> {
        let log_entries = &self.log_entries;
        let scroll_offset = self.scroll_offset;
        let selected_index = self.selected_index;
        let auto_scroll = self.auto_scroll;

        self.terminal.draw(|frame| {
            Self::draw_ui_static(
                frame,
                log_entries,
                scroll_offset,
                selected_index,
                auto_scroll,
            );
        })?;
        Ok(())
    }

    /// Draw the user interface (static version to avoid borrowing issues)
    fn draw_ui_static(
        frame: &mut Frame,
        log_entries: &[LogEntry],
        scroll_offset: usize,
        selected_index: Option<usize>,
        auto_scroll: bool,
    ) {
        let size = frame.area();

        // Create layout
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3), // Header
                Constraint::Min(0),    // Log content
                Constraint::Length(3), // Footer
            ])
            .split(size);

        // Header
        let header_text = if auto_scroll {
            "Log Viewer - Press 'q' to quit, arrow keys to scroll, 'c' to clear, 'f' to toggle follow [FOLLOWING]"
        } else {
            "Log Viewer - Press 'q' to quit, arrow keys to scroll, 'c' to clear, 'f' to toggle follow [PAUSED]"
        };

        let header = Paragraph::new(header_text)
            .block(Block::default().borders(Borders::ALL).title("Controls"))
            .style(Style::default().fg(Color::Cyan));
        frame.render_widget(header, chunks[0]);

        // Log entries
        Self::draw_log_entries_static(frame, chunks[1], log_entries, scroll_offset, selected_index);

        // Footer with status
        let terminal_height = frame.area().height as usize;
        let content_height = terminal_height.saturating_sub(4);
        let status = format!(
            "Entries: {} | Scroll: {} | Screen: {} | Mode: {} | Use ↑↓/j/k, PgUp/PgDn, Home/End to navigate",
            log_entries.len(),
            scroll_offset,
            content_height,
            if auto_scroll { "Following" } else { "Paused" }
        );
        let footer = Paragraph::new(status)
            .block(Block::default().borders(Borders::ALL).title("Status"))
            .style(Style::default().fg(Color::Yellow));
        frame.render_widget(footer, chunks[2]);
    }

    /// Draw the log entries list (static version to avoid borrowing issues)
    fn draw_log_entries_static(
        frame: &mut Frame,
        area: Rect,
        log_entries: &[LogEntry],
        scroll_offset: usize,
        selected_index: Option<usize>,
    ) {
        let terminal_height = area.height as usize;
        let content_height = terminal_height.saturating_sub(2); // Account for borders

        let visible_entries: Vec<ListItem> = log_entries
            .iter()
            .skip(scroll_offset)
            .take(content_height)
            .enumerate()
            .map(|(i, entry)| {
                let content = entry.content.clone();
                let style = if Some(i + scroll_offset) == selected_index {
                    Style::default()
                        .bg(Color::DarkGray)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };

                ListItem::new(Line::from(Span::styled(content, style)))
            })
            .collect();

        let list = List::new(visible_entries)
            .block(Block::default().borders(Borders::ALL).title("Log Entries"))
            .style(Style::default().fg(Color::White));

        frame.render_widget(list, area);

        // Render scrollbar if needed
        if log_entries.len() > content_height {
            let mut scrollbar_state = ScrollbarState::default()
                .content_length(log_entries.len())
                .viewport_content_length(content_height)
                .position(scroll_offset);

            let scrollbar = Scrollbar::default()
                .orientation(ScrollbarOrientation::VerticalRight)
                .begin_symbol(Some("↑"))
                .end_symbol(Some("↓"));

            frame.render_stateful_widget(
                scrollbar,
                area.inner(ratatui::layout::Margin {
                    vertical: 1,
                    horizontal: 0,
                }),
                &mut scrollbar_state,
            );
        }
    }

    /// Run the main TUI loop with optional callback for external events
    pub fn run_loop<F>(&mut self, mut external_event_handler: F) -> io::Result<()>
    where
        F: FnMut(&mut Self) -> io::Result<bool>,
    {
        loop {
            self.render()?;

            // Handle TUI input
            if !self.handle_input()? {
                break;
            }

            // Handle external events (like file changes)
            if !external_event_handler(self)? {
                break;
            }
        }
        Ok(())
    }
}
