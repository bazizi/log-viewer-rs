use anyhow::Result;
use std::io::Read;

use log::{error, info};

use regex::Regex;

pub type LogEntry = Vec<String>;

lazy_static! {
    static ref REGEX_PATTERNS : Vec<Regex> = vec![
        Regex::new(r#"^\s*[^\[]+\[(?P<date>\d{2}:\d{2}:\d{2}:\d+)\]:\s*(?P<log>.*)$"#).unwrap(),                                                        // Windows installer (MSI)
        Regex::new(r#"^\s*(?P<id>\d+)\s+\[(?P<date>[^\]]+)\]\s+PID:\s*(?P<pid>\d+)\s+TID:\s*(?P<tid>\d+)\s+(?P<level>\w+)\s+(?P<log>.*)$"#).unwrap(),   // EA app
        Regex::new(r#"^\s*\[[^\]]+\]\[(?P<date>\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2})\]i\d+:\s*(?P<log>.*)$"#).unwrap(),                                  // EA app vc_redist
        Regex::new(r#"^\s*(?P<level>\w+)\s+(?P<date>\d{2}:\d{2}:\d{2}\s+\w+)\s+\(\s+\d+\)\s+(?P<tid>\d+)\s+(?P<log>.*)$"#).unwrap(),                    // EA app IGO
        Regex::new(r#"^\s*(?P<level>\w+)\s+(?P<date>\d{2}:\d{2}:\d{2}\s+\w+)\s+(?P<tid>\d+)\s+\s+(?P<log>.*)$"#).unwrap(),                              // EA app IGO Proxy
        Regex::new(r#"^\s*\[(?P<date>\d{4}-\d{2}-\d{2}\s\d{2}:\d{2}:\d{2})\]\s+(?P<log>.*)$"#).unwrap(),                                                // Steam
        Regex::new(r#"^\s*\[(?P<date>\d{4}\.\d{2}\.\d{2}-\d{2}\.\d{2}\.\d{2}:\d+)\][^\]]+\](?P<log>.*)$"#).unwrap(),                                    // Riot launcher (Valorant) + Epic games
        Regex::new(r#"^\s*\[(?P<date>[^:]+):(?P<level>\w+):[^\]]+\]\s*(?P<log>.*)$"#).unwrap(),                                                         // CEF
    ];
}

pub enum LogEntryIndices {
    FileName,
    // _ID,
    Date,
    // _PID,
    // _TID,
    Level,
    Log,
}

pub fn parse_log_by_path(log_path: &str) -> Result<Vec<LogEntry>> {
    info!("Attempting to parse log file [{}]...", log_path);

    let mut line_num = 0;
    let mut _session = 0;
    let mut contents = String::new();
    let lines = {
        let mut f = std::fs::File::open(log_path)?;
        f.read_to_string(&mut contents)?;
        contents.lines().collect::<Vec<&str>>()
    };

    let mut log_entries = Vec::<Vec<String>>::new();
    while line_num < lines.len() {
        let mut log = String::new();
        let line = lines[line_num];
        if line.is_empty() {
            line_num += 1;
            continue;
        }

        let mut captures = None;
        for regex_pattern in REGEX_PATTERNS.iter() {
            let mut captures_iter = regex_pattern.captures_iter(line);

            if let Some(tmp) = captures_iter.next() {
                captures = Some(tmp);
                break;
            }
        }

        if captures.is_none() {
            error!("Error parsinig line: [{}]", line);
            line_num += 1;
            continue;
        }

        let captures = captures.unwrap();
        let _id = line_num;
        if captures.name("id").map_or("", |m| m.as_str()) == "0" {
            _session += 1;
        }
        let date = &captures.name("date").map_or("", |m| m.as_str());
        let _pid = &captures.name("pid").map_or("", |m| m.as_str());
        let _tid = &captures.name("tid").map_or("", |m| m.as_str());
        let level = &captures.name("level").map_or("", |m| m.as_str());
        log += &captures.name("log").map_or("", |m| m.as_str());

        loop {
            // Deal with multiline log entries where only the 1st line matches the regex.
            // We append the next lines to the first line and show them as a single log entry
            if line_num >= lines.len() - 1 {
                break;
            }

            line_num += 1;
            let next_line = lines[line_num];

            let mut valid_captures = None;
            for regex_pattern in REGEX_PATTERNS.iter() {
                let mut captures = regex_pattern.captures_iter(next_line);

                if captures.next().is_some() {
                    valid_captures = Some(captures);
                    break;
                }
            }

            if valid_captures.is_none() {
                // Current line doesn't match any known formats so we assume it's a continuation of a multiline log entry
                log += next_line;
                continue;
            }

            // Current line is an actual log line (and not a continuation of a multiline log entry)
            // So we go back to the previous line and break (so that the current line will be processed as a separate entry)
            line_num -= 1;
            break;
        }

        log_entries.push(vec![
            std::path::Path::new(log_path)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string(),
            // id.to_string(),
            // _session.to_string(),
            date.to_string(),
            // pid.to_string(),
            // tid.to_string(),
            level.to_string(),
            log.to_string(),
        ]);

        line_num += 1;
    }

    info!(
        "found [{}] log _sessions in [{}]",
        log_entries.len(),
        log_path
    );

    Ok(log_entries)
}
