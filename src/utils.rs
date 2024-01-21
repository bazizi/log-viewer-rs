use ratatui::prelude::Line;
use ratatui::prelude::Modifier;
use ratatui::prelude::Span;
use ratatui::prelude::Style;
use ratatui::prelude::Text;
use ratatui::style::Stylize;
use serde_json::Value;

#[derive(Copy, Clone)]
enum TextStyle {
    None,
    Highlight,
    Digit,
    Separator,
}

fn highlight_search_matches<'a>(text: &'a str, keywords: &'a str) -> Vec<(String, TextStyle)> {
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
        text_spans.push((
            text[prev_span_end..keyword_start].to_owned(),
            TextStyle::None,
        ));

        text_spans.push((
            text[keyword_start..keyword_end].to_owned(),
            TextStyle::Highlight,
        ));

        prev_span_end = keyword_end;
    }

    text_spans.push((text[prev_span_end..].to_owned(), TextStyle::None));
    text_spans
}

fn highlight_chars<F>(
    condition: F,
    spans: Vec<(String, TextStyle)>,
    highlight_type: TextStyle,
) -> Vec<(String, TextStyle)>
where
    F: Fn(char) -> bool,
{
    let mut spans_ret = vec![];
    for span in spans {
        if let TextStyle::Highlight = span.1 {
            // Avoid messing with highlighted items
            spans_ret.push((span.0.to_owned(), span.1));
            continue;
        }

        let mut numeric_char_indices = vec![];
        for (index, chr) in span.0.char_indices() {
            if condition(chr) {
                numeric_char_indices.push(index);
            }
        }

        let mut last_index_end = 0;
        for index in numeric_char_indices {
            spans_ret.push((span.0[last_index_end..index].to_owned(), span.1));

            spans_ret.push((span.0[index..index + 1].to_owned(), highlight_type));
            last_index_end = index + 1;
        }

        spans_ret.push((span.0[last_index_end..].to_owned(), span.1));
    }

    spans_ret
}

pub fn highlight_keywords_in_text<'a>(text: &'a str, keywords: &'a str) -> Text<'a> {
    let text_spans = highlight_search_matches(text, keywords);
    let mut text_spans = highlight_chars(|chr| chr.is_numeric(), text_spans, TextStyle::Digit);
    text_spans = highlight_chars(
        |chr| {
            let chars = "/\\-:,\".[](){}";
            for sep_chr in chars.chars() {
                if chr == sep_chr {
                    return true;
                }
            }
            false
        },
        text_spans,
        TextStyle::Separator,
    );

    let mut line_spans = vec![];
    for span in text_spans {
        let lines = span.0.lines();
        let mut is_new_line = false;
        for line in lines {
            line_spans.push((
                line.to_owned(),
                span.1,
                is_new_line, /* controls whether span should go on a new line */
            ));
            is_new_line = true;
        }
    }

    let mut spans = line_spans
        .into_iter()
        .map(|span| {
            if let TextStyle::Highlight = span.1 {
                return Some((
                    Span::styled(
                        span.0.to_owned(),
                        Style::new().add_modifier(Modifier::BOLD).on_light_magenta(),
                    ),
                    span.2,
                ));
            }

            if let TextStyle::Digit = span.1 {
                return Some((
                    Span::styled(
                        span.0.to_owned(),
                        Style::new().fg(ratatui::style::Color::LightGreen),
                    ),
                    span.2,
                ));
            }

            if let TextStyle::Separator = span.1 {
                return Some((
                    Span::styled(
                        span.0.to_owned(),
                        Style::new().fg(ratatui::style::Color::LightCyan),
                    ),
                    span.2,
                ));
            }
            Some((Span::raw(span.0.to_owned()), span.2))
        })
        .collect::<Vec<Option<(Span, bool)>>>();

    let lines = {
        let mut span_index = 0;
        let mut lines = vec![];
        loop {
            if span_index >= spans.len() {
                break;
            }

            let mut spans_on_same_line = vec![];

            loop {
                spans_on_same_line.push(spans[span_index].take().unwrap().0);
                span_index += 1;
                if span_index >= spans.len() || spans[span_index].as_ref().unwrap().1 {
                    break;
                }
            }

            lines.push(Line::from(spans_on_same_line));
        }
        lines
    };

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

