// src/tui/components.rs
use crate::app::AppState;
use crate::models::MessageStatus;
use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render_messages<'a>(state: &AppState) -> (List<'a>, String) {
    let self_user = &state.self_username;
    let show_time = state.show_timestamp;
    let show_lat = state.show_latency;

    let msgs: Vec<ListItem> = state.messages.iter().map(|m| {
        let is_me = m.author == *self_user;
        let author_color = if is_me { Color::Blue } else { Color::Green };
        let header_style = Style::default().fg(author_color);

        let content_color = match m.status {
            MessageStatus::Sending => Color::DarkGray,
            MessageStatus::Failed => Color::Red,
            MessageStatus::Delivered => Color::White,
        };

        let status_indicator = match m.status {
            MessageStatus::Sending => " [...]",
            MessageStatus::Failed => " [❌]",
            MessageStatus::Delivered => "",
        };

        let header_line = if show_time || show_lat {
            let mut parts = m.timestamp.split('|');
            let time_part = parts.next().unwrap_or("").trim();
            let lat_part = parts.next().unwrap_or("").trim();

            let meta_str = match (show_time && !time_part.is_empty(), show_lat && !lat_part.is_empty()) {
                (true, true) => format!("[{} | {}]", time_part, lat_part),
                (true, false) => format!("[{}]", time_part),
                (false, true) => format!("[{}]", lat_part),
                (false, false) => String::new(),
            };

            if !meta_str.is_empty() {
                Line::from(vec![
                    Span::styled(&m.author, header_style),
                    Span::raw(" "),
                    Span::styled(meta_str, header_style),
                    Span::styled(status_indicator, header_style),
                ])
            } else {
                Line::from(vec![
                    Span::styled(&m.author, header_style),
                    Span::styled(status_indicator, header_style),
                ])
            }
        } else {
            Line::from(vec![
                Span::styled(&m.author, header_style),
                Span::styled(status_indicator, header_style),
            ])
        };

        let content_line = Line::from(vec![
            Span::raw("  "),
            Span::styled(&m.content, Style::default().fg(content_color)),
        ]);

        ListItem::new(vec![header_line, content_line])
    }).collect();

    let time_status = if show_time { "F2: Hide Time" } else { "F2: Show Time" };
    let lat_status = if show_lat { "F3: Hide Latency" } else { "F3: Show Latency" };
    let title_text = format!(" messages [{} | {} | Ctrl+G/F5: Channel | Ctrl+X/F4: Switch Token] ", time_status, lat_status);

    let msg_list = List::new(msgs)
        .block(Block::default()
            .borders(Borders::ALL)
            .title(title_text));

    (msg_list, state.input_text.clone())
}

pub fn render_input_box<'a>(input_text: &'a str) -> Paragraph<'a> {
    let prompt_span = Span::raw("> ");
    let text_span = Span::styled(
        input_text,
        Style::default().fg(Color::Yellow)
    );
    let input_line = Line::from(vec![prompt_span, text_span]);

    Paragraph::new(input_line)
        .block(Block::default().borders(Borders::ALL))
}
