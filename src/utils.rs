use ratatui::prelude::Line;
use ratatui::prelude::Modifier;
use ratatui::prelude::Span;
use ratatui::prelude::Style;
use ratatui::prelude::Text;
use ratatui::style::Stylize;
use serde_json::Value;

pub fn highlight_keywords_in_text<'a>(text: &'a String, keywords: &'a String) -> Text<'a> {
    let keywords = keywords
        .split(',')
        .map(|keyword| keyword.to_owned())
        .collect::<Vec<String>>();

    let mut keyword_positions_in_text = vec![];

    {
        let mut last_keyword_end_absolute = 0;
        for keyword in keywords {
            if keyword.is_empty() {
                continue;
            }

            if let Some(keyword_begin_relative) = text[last_keyword_end_absolute..]
                .to_lowercase()
                .find(&keyword.to_lowercase())
            {
                let keyword_begin_absolute = last_keyword_end_absolute + keyword_begin_relative;
                last_keyword_end_absolute = keyword_begin_absolute + keyword.len();
                keyword_positions_in_text.push((keyword_begin_absolute, last_keyword_end_absolute));
            }
        }
    }

    let mut text_spans = vec![];
    let mut prev_span_end = 0;
    for (keyword_start, keyword_end) in keyword_positions_in_text {
        text_spans.push(Span::raw(&text[prev_span_end..keyword_start]));

        text_spans.push(Span::styled(
            &text[keyword_start..keyword_end],
            Style::new().add_modifier(Modifier::BOLD).on_light_magenta(),
        ));

        prev_span_end = keyword_end;
    }

    text_spans.push(Span::raw(&text[prev_span_end..]));

    Text::from(Line::from(text_spans))
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
