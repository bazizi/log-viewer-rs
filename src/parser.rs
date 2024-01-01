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

fn parse_log_vec(lines: &Vec<&str>, log_path: &str) -> Vec<Vec<String>> {
    let mut line_num = 0;
    let mut _session = 0;
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
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default()
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

    log_entries
}

pub fn parse_log_by_path(log_path: &str) -> Result<Vec<LogEntry>> {
    info!("Attempting to parse log file [{}]...", log_path);

    let mut contents = String::new();
    let lines = {
        let mut f = std::fs::File::open(log_path)?;
        f.read_to_string(&mut contents)?;
        contents.lines().collect::<Vec<&str>>()
    };

    Ok(parse_log_vec(&lines, log_path))
}

#[cfg(test)]
mod tests {
    use crate::parser::parse_log_vec;

    #[test]
    fn test_msi_parse() {
        let log_lines = vec![
            "an invalid line",
            "[1E78:1CCC][2023-09-03T16:46:28]i001: Burn v3.8.1128.0, Windows v6.3 (Build 9600: Service Pack 0), path: C:\\Program Files (x86)\\Epic Games\\Launcher\\Portal\\SelfUpdateStaging\\Install\\Portal\\Extras\\Redist\\LauncherPrereqSetup_x64.exe, cmdline: '/quiet /log \"C:/Users/behna/AppData/Local/EpicGamesLauncher/Saved/Logs/SelfUpdatePrereqInstall.log\" -burn.unelevated BurnPipe.{728D72EE-4979-4A05-84E7-FCB1B3712CDD} {529A9DC5-F441-4AF4-ACBC-8809762531C3} 20096'",
            "another invalid line",
            "[1E78:1CCC][2023-09-03T16:46:28]i000: Setting string variable 'WixBundleLog' to value 'C:/Users/behna/AppData/Local/EpicGamesLauncher/Saved/Logs/SelfUpdatePrereqInstall.log'",
            "another invalid line",
        ];
        assert_eq!(parse_log_vec(&log_lines, "").len(), 2);
    }

    #[test]
    fn test_cef_parse() {
        let log_lines = vec![
            "an invalid line",
            "[0901/211250.717:ERROR:adm_helpers.cc(62)] Failed to query stereo recording.",
            "an invalid line",
            "[0901/211250.738:WARNING:mediasession.cc(347)] Duplicate id found. Reassigning from 104 to 125",
            "an invalid line",
            "[0901/211250.798:WARNING:stunport.cc(384)] Jingle:Port[000002172D841260:data:1:0:local:Net[any:0:0:0:x:x:x:x:x/0:Unknown]]: StunPort: stun host lookup received error 0",
            "an invalid line",
            "[1013/215308.845:ERROR:adm_helpers.cc(62)] Failed to query stereo recording.",
            "an invalid line",
            "[1013/215308.863:WARNING:mediasession.cc(347)] Duplicate id found. Reassigning from 104 to 125",
            "an invalid line",
            "[1013/215308.921:WARNING:stunport.cc(384)] Jingle:Port[0000018F02628910:data:1:0:local:Net[any:0:0:0:x:x:x:x:x/0:Unknown]]: StunPort: stun host lookup received error 0",
            "an invalid line",
        ];
        assert_eq!(parse_log_vec(&log_lines, "").len(), 6);
    }
    #[test]
    fn test_eaa_parse() {
        // TODO - add test
    }
}
