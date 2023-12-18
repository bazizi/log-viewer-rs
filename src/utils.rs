use ratatui::prelude::Line;
use ratatui::prelude::Modifier;
use ratatui::prelude::Span;
use ratatui::prelude::Style;
use ratatui::prelude::Text;
use ratatui::style::Stylize;
use serde_json::Value;

pub fn highlight_keyword_in_text<'a>(text: &'a String, keyword: &'a String) -> Text<'a> {
    let lines: Vec<Line> = text
        .lines()
        .map(|line| {
            let mut line = Line::from(vec![Span::raw(line)]);
            if let Some(bold_begin) = text.to_lowercase().find(&keyword.to_lowercase()) {
                let bold_end = bold_begin + keyword.len();
                let span1 = Span::raw(&text[0..bold_begin]);
                let span2 = Span::styled(
                    &text[bold_begin..bold_end],
                    Style::new().add_modifier(Modifier::BOLD).on_light_magenta(),
                );
                let span3 = Span::raw(&text[bold_end..text.len()]);
                line = Line::from(vec![span1, span2, span3]);
            }

            line
        })
        .collect();
    Text::from(lines)
}

pub fn beatify_enclosed_json(log: &str) -> Option<String> {
    if let (Some(first_curly), Some(last_curly)) = (log.find('{'), log.rfind('}')) {
        let json_part = &log[first_curly..last_curly + 1];
        if let Ok(value) = serde_json::from_str::<Value>(json_part.to_string().as_str()) {
            if let Ok(pretty_str) = serde_json::to_string_pretty(&value) {
                return Some(
                    log[0..first_curly].to_owned() + &pretty_str + &log[last_curly..log.len()],
                );
            }
        }
    }
    None
}
