//! `cargo run --example album_view` — press `f` to flip between cover art and tracklist, `n` for the next album, `q` to quit.
//!
//! A "vinyl record" style demo: Side A shows now-playing with cover art, Side B shows the full tracklist.

use std::io;
use std::time::Duration;

use crossterm::event::{self, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};

use ratatui::backend::CrosstermBackend;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};
use ratatui::Terminal;

use ratatui_flip_panel::{FlipPanel, FlipState};

struct Album {
    title: &'static str,
    artist: &'static str,
    year: u16,
    art: &'static [&'static str],
    tracks: &'static [(&'static str, &'static str)],
    accent: Color,
}

const ALBUMS: &[Album] = &[
    Album {
        title: "MIDNIGHT TERMINAL",
        artist: "The Daemons",
        year: 2026,
        accent: Color::Magenta,
        art: &[
            "    ▄▄▄▄▄▄▄▄▄▄▄    ",
            "  ▄█████████████▄  ",
            " ███▀         ▀███ ",
            "███   ▄▀▀▀▀▄   ███",
            "███  ▐ ◉  ◉ ▌  ███",
            "███   ▀▄▄▄▄▀   ███",
            " ███▄         ▄███ ",
            "  ▀█████████████▀  ",
            "    ▀▀▀▀▀▀▀▀▀▀▀    ",
        ],
        tracks: &[
            ("1.", "Boot Sequence"),
            ("2.", "Kernel Panic"),
            ("3.", "Race Condition (feat. Mutex)"),
            ("4.", "Stack Overflow"),
            ("5.", "Segfault Lullaby"),
            ("6.", "Garbage Collector"),
            ("7.", "Endless Loop"),
        ],
    },
    Album {
        title: "ASCII DREAMS",
        artist: "Block Renderer",
        year: 2025,
        accent: Color::Cyan,
        art: &[
            "  ░░░▒▒▒▓▓▓███▓▓▓▒▒▒░░░  ",
            " ░▒▓█████████████████▓▒░ ",
            "░▓███▀  ░░░░░░░░░  ▀███▓░",
            "▒██▀  ░  ▄▄▄▄▄  ░  ▀██▒",
            "▓██  ░  █     █  ░  ██▓",
            "▒██▄  ░  ▀▀▀▀▀  ░  ▄██▒",
            "░▓███▄  ░░░░░░░░░  ▄███▓░",
            " ░▒▓█████████████████▓▒░ ",
            "  ░░░▒▒▒▓▓▓███▓▓▓▒▒▒░░░  ",
        ],
        tracks: &[
            ("1.", "Bitmap Sunrise"),
            ("2.", "Halftone Heart"),
            ("3.", "Dithered Sky"),
            ("4.", "Quadrant Blues"),
            ("5.", "Block Element Anthem"),
        ],
    },
];

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FlipState::new(Duration::from_millis(400));
    let mut idx: usize = 0;

    loop {
        let album = &ALBUMS[idx];

        terminal.draw(|frame| {
            let area = centered(frame.area(), 52, 16);
            let widget = FlipPanel::new(
                |a, b| draw_cover(album, a, b),
                |a, b| draw_tracks(album, a, b),
            );
            frame.render_stateful_widget(widget, area, &mut state);
            frame.render_widget(hint(), bottom(frame.area()));
        })?;

        let timeout = state.next_tick_in().unwrap_or(Duration::from_secs(1));
        if event::poll(timeout)? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('f') => {
                        state.flip();
                    }
                    KeyCode::Char('n') => {
                        idx = (idx + 1) % ALBUMS.len();
                        state.set_showing_back(false);
                    }
                    _ => {}
                }
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}

fn centered(area: Rect, w: u16, h: u16) -> Rect {
    let w = w.min(area.width);
    let h = h.min(area.height);
    Rect::new(
        area.x + (area.width - w) / 2,
        area.y + (area.height - h) / 2,
        w,
        h,
    )
}

fn bottom(area: Rect) -> Rect {
    Rect::new(
        area.x,
        area.y + area.height.saturating_sub(2),
        area.width,
        1,
    )
}

fn draw_cover(album: &Album, area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(album.accent))
        .title(Span::styled(
            " ♫ NOW PLAYING ",
            Style::default()
                .fg(album.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 1, area.width - 4, area.height - 2);
    let mut lines: Vec<Line> = album
        .art
        .iter()
        .map(|row| Line::from(Span::styled(*row, Style::default().fg(album.accent))).centered())
        .collect();
    lines.push(Line::from(""));
    lines.push(
        Line::from(Span::styled(
            album.title,
            Style::default()
                .fg(Color::White)
                .add_modifier(Modifier::BOLD),
        ))
        .centered(),
    );
    lines.push(
        Line::from(Span::styled(
            format!("{}  ·  {}", album.artist, album.year),
            Style::default().fg(Color::Gray),
        ))
        .centered(),
    );
    Paragraph::new(lines).render(inner, buf);
}

fn draw_tracks(album: &Album, area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(album.accent))
        .title(Span::styled(
            " ▶ TRACKS ",
            Style::default()
                .fg(album.accent)
                .add_modifier(Modifier::BOLD),
        ))
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);
    let header = Line::from(Span::styled(
        format!("{}  ·  {}  ·  {}", album.artist, album.title, album.year),
        Style::default().fg(Color::Gray),
    ));
    let mut lines = vec![header, Line::from("")];
    for (num, title) in album.tracks {
        lines.push(Line::from(vec![
            Span::styled(
                *num,
                Style::default()
                    .fg(album.accent)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw("  "),
            Span::styled(*title, Style::default().fg(Color::White)),
        ]));
    }
    Paragraph::new(lines).render(inner, buf);
}

fn hint() -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled("[f]", Style::default().fg(Color::Yellow)),
        Span::raw(" flip   "),
        Span::styled("[n]", Style::default().fg(Color::Yellow)),
        Span::raw(" next album   "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
}
