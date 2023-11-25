use anyhow::Result;
use std::io::Read;
use std::io::Seek;

use log::{error, info};

pub type LogEntry = Vec<String>;

pub enum LogEntryIndices {
    // _ID,
    _DATE,
    // _PID,
    _TID,
    LEVEL,
    LOG,
}

pub fn parse_log_by_path(log_path: &str, start_offset: u64) -> Result<Vec<LogEntry>> {
    info!("Attempting to parse log file [{}]...", log_path);
    let re = regex::Regex::new(
        r#"^\s+(?P<id>\d+)\s+\[(?P<date>[^\]]+)\]\s+PID:\s*(?P<pid>\d+)\s+TID:\s*(?P<tid>\d+)\s+(?P<level>\w+)\s+(?P<log>.*)"#,
    )?;

    let mut line_num = 0;
    let mut _session = 0;
    let mut contents = String::new();
    let lines = {
        let mut f = std::fs::File::open(log_path)?;
        f.seek(std::io::SeekFrom::Start(start_offset))?;
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

        let mut captures = re.captures_iter(&line);

        let cap;
        if let Some(tmp) = captures.next() {
            cap = tmp;
        } else {
            error!("Error parsinig line: [{}]", line);
            line_num += 1;
            continue;
        }

        if &cap["id"] == "0" {
            _session += 1;
        }

        let id = line_num;
        let date = &cap["date"];
        let pid = &cap["pid"];
        let tid = &cap["tid"];
        let level = &cap["level"];
        log += &cap["log"];

        loop {
            /* Deal with multiline log entries. For example:
                1291	[2023-05-23T19:54:39.779Z]	PID: 13052	TID: 15836	VERBOSE 	(eax::services::localStorage::encryptDataToFile)	Saving [IQ] into file:
                [C:\ProgramData\EA Desktop\530c11479fe252fc5aabc24935b9776d4900eb3ba58fdc271e0d6229413ad40e\IQ]
            */

            if line_num >= lines.len() - 1 {
                break;
            }

            line_num += 1;
            let next_line = lines[line_num];
            let mut cap = re.captures_iter(&next_line);
            if cap.next().is_none() {
                log += next_line;
                continue;
            }

            line_num -= 1;
            break;
        }

        log_entries.push(vec![
            // id.to_string(),
            // _session.to_string(),
            date.to_string(),
            // pid.to_string(),
            tid.to_string(),
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

    return Ok(log_entries);
}
