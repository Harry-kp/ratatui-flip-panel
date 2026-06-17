//! `cargo run --example playing_card` — press `f` to flip, `n` for a new card, `q` to quit.
//!
//! A deck-of-cards demo: every flip reveals a different randomly picked card.

#![allow(clippy::cast_possible_truncation)]

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

const RANKS: &[&str] = &[
    "A", "2", "3", "4", "5", "6", "7", "8", "9", "10", "J", "Q", "K",
];
const SUITS: &[(&str, Color)] = &[
    ("♠", Color::White),
    ("♥", Color::Red),
    ("♦", Color::Red),
    ("♣", Color::White),
];

fn pick(seed: u64) -> (&'static str, &'static str, Color) {
    let mut h = DefaultHasher::new();
    seed.hash(&mut h);
    let n = h.finish();
    let rank = RANKS[(n as usize) % RANKS.len()];
    let (suit, color) = SUITS[((n >> 8) as usize) % SUITS.len()];
    (rank, suit, color)
}

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FlipState::new(Duration::from_millis(450));
    let mut seed: u64 = 7;

    loop {
        let (rank, suit, color) = pick(seed);

        terminal.draw(|frame| {
            let card = centered(frame.area(), 22, 14);
            let widget = FlipPanel::new(draw_back, |a, b| draw_face(rank, suit, color, a, b));
            frame.render_stateful_widget(widget, card, &mut state);
            frame.render_widget(hint(), bottom(frame.area()));
        })?;

        let timeout = state.next_tick_in().unwrap_or(Duration::from_secs(1));
        if event::poll(timeout)? {
            if let Event::Key(k) = event::read()? {
                match k.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    KeyCode::Char('f') if state.flip() => seed = seed.wrapping_add(1),
                    KeyCode::Char('n') => seed = seed.wrapping_add(1),
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

fn draw_back(area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Blue))
        .render(area, buf);
    if area.width <= 2 || area.height <= 2 {
        return;
    }
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let row = "◆ ".repeat(usize::from(inner.width / 2));
    let lines: Vec<Line> = (0..inner.height)
        .map(|_| Line::from(Span::styled(row.clone(), Style::default().fg(Color::Blue))))
        .collect();
    Paragraph::new(lines).render(inner, buf);
}

fn draw_face(rank: &str, suit: &str, color: Color, area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(color))
        .render(area, buf);
    if area.width <= 2 || area.height <= 2 {
        return;
    }
    let inner = Rect::new(area.x + 1, area.y + 1, area.width - 2, area.height - 2);
    let bold = Style::default().fg(color).add_modifier(Modifier::BOLD);
    let mut lines = vec![Line::from(Span::styled(format!("{rank}{suit}"), bold))];
    let pad = inner.height.saturating_sub(3) / 2;
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(format!("{rank}{suit}"), bold)).centered());
    for _ in 0..pad {
        lines.push(Line::from(""));
    }
    lines.push(Line::from(Span::styled(format!("{suit}{rank}"), bold)).right_aligned());
    Paragraph::new(lines).render(inner, buf);
}

fn hint() -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled("[f]", Style::default().fg(Color::Yellow)),
        Span::raw(" flip   "),
        Span::styled("[n]", Style::default().fg(Color::Yellow)),
        Span::raw(" new card   "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
}
