//! `cargo run --example flashcard` — press `f` to reveal the answer, `n` for the next card, `q` to quit.
//!
//! A small Rust-trivia deck. The flip metaphor maps naturally onto study cards: question → answer.

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
use ratatui::widgets::{Block, Borders, Paragraph, Widget, Wrap};
use ratatui::Terminal;

use ratatui_flip_panel::{FlipPanel, FlipState};

#[derive(Clone, Copy)]
struct Card {
    question: &'static str,
    answer: &'static str,
    tag: &'static str,
    color: Color,
}

const DECK: &[Card] = &[
    Card {
        question: "What lifetime annotation lets a function return a reference tied to one of its inputs?",
        answer: "An explicit lifetime parameter, e.g. fn f<'a>(s: &'a str) -> &'a str.",
        tag: "easy",
        color: Color::Green,
    },
    Card {
        question: "Why doesn't &T: Copy imply T: Copy?",
        answer: "Shared references are always Copy regardless of T — the bytes copied are the pointer, not the pointee.",
        tag: "medium",
        color: Color::Yellow,
    },
    Card {
        question: "What's the difference between Box<dyn Trait> and impl Trait in a return position?",
        answer: "Box<dyn Trait> is dynamic dispatch (one heap alloc, one vtable lookup per call). impl Trait is static dispatch (monomorphised, but the concrete type must be the same on every return path).",
        tag: "medium",
        color: Color::Yellow,
    },
    Card {
        question: "Why is Pin<&mut T> required for self-referential futures?",
        answer: "Moving a struct that holds a pointer into itself would invalidate the internal pointer. Pin guarantees the value won't be moved.",
        tag: "hard",
        color: Color::Red,
    },
    Card {
        question: "Name three ways `?` can be customised.",
        answer: "Implementing From for the source error type, the Try trait (nightly), or wrapping with .map_err(|e| ...)? at the call site.",
        tag: "medium",
        color: Color::Yellow,
    },
    Card {
        question: "What does #[repr(transparent)] guarantee?",
        answer: "A single-field newtype has the same memory layout, ABI, and FFI representation as its inner type.",
        tag: "hard",
        color: Color::Red,
    },
];

fn main() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut state = FlipState::new(Duration::from_millis(350));
    let mut idx: usize = 0;

    loop {
        let card = DECK[idx];
        let card_n = idx + 1;
        let total = DECK.len();

        terminal.draw(|frame| {
            let area = centered(frame.area(), 60, 14);
            let widget = FlipPanel::new(
                |a, b| draw_front(card, card_n, total, a, b),
                |a, b| draw_back(card, a, b),
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
                        idx = (idx + 1) % DECK.len();
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

fn draw_front(card: Card, n: usize, total: usize, area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .title(format!(" Question {n} / {total} "))
        .title_style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);
    Paragraph::new(card.question)
        .style(Style::default().add_modifier(Modifier::BOLD))
        .wrap(Wrap { trim: true })
        .render(inner, buf);
}

fn draw_back(card: Card, area: Rect, buf: &mut Buffer) {
    Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(card.color))
        .title(Line::from(vec![
            Span::raw(" Answer "),
            Span::styled(
                format!(" {} ", card.tag),
                Style::default()
                    .fg(Color::Black)
                    .bg(card.color)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::raw(" "),
        ]))
        .render(area, buf);
    if area.width <= 4 || area.height <= 4 {
        return;
    }
    let inner = Rect::new(area.x + 2, area.y + 2, area.width - 4, area.height - 4);
    Paragraph::new(card.answer)
        .style(Style::default().fg(Color::White))
        .wrap(Wrap { trim: true })
        .render(inner, buf);
}

fn hint() -> Paragraph<'static> {
    Paragraph::new(Line::from(vec![
        Span::styled("[f]", Style::default().fg(Color::Yellow)),
        Span::raw(" reveal   "),
        Span::styled("[n]", Style::default().fg(Color::Yellow)),
        Span::raw(" next card   "),
        Span::styled("[q]", Style::default().fg(Color::Yellow)),
        Span::raw(" quit"),
    ]))
    .alignment(Alignment::Center)
}
