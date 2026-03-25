use serde::Serialize;

use crate::usb::{Report, Status};

#[derive(Debug, Serialize)]
struct JsonCheck<'a> {
    status: &'a str,
    check_id: &'a str,
    message: &'a str,
    hint: Option<&'a str>,
}

#[derive(Debug, Serialize)]
struct JsonEvent<'a> {
    kind: &'a str,
    text: &'a str,
}

#[derive(Debug, Serialize)]
struct JsonReport<'a> {
    schema_version: &'a str,
    overall: &'a str,
    checks: Vec<JsonCheck<'a>>,
    events: Vec<JsonEvent<'a>>,
}

pub fn render_text(report: &Report, verbose: bool) -> String {
    let mut lines = Vec::new();
    for check in &report.checks {
        lines.push(render_check_line(
            check.status,
            &check.check_id,
            &check.message,
            check.hint.as_deref(),
        ));
    }
    if verbose {
        for event in &report.events {
            lines.push(format!("TRACE {}: {}", event.kind, event.text));
        }
    }
    lines.join("\n")
}

pub fn render_json(report: &Report) -> Result<String, serde_json::Error> {
    let json = JsonReport {
        schema_version: &report.schema_version,
        overall: report.overall.as_str(),
        checks: report
            .checks
            .iter()
            .map(|check| JsonCheck {
                status: check.status.as_str(),
                check_id: &check.check_id,
                message: &check.message,
                hint: check.hint.as_deref(),
            })
            .collect(),
        events: report
            .events
            .iter()
            .map(|event| JsonEvent {
                kind: &event.kind,
                text: &event.text,
            })
            .collect(),
    };
    serde_json::to_string(&json)
}

fn render_check_line(status: Status, check_id: &str, message: &str, hint: Option<&str>) -> String {
    if let Some(h) = hint {
        format!("{} {}: {} (hint: {h})", status.as_str(), check_id, message)
    } else {
        format!("{} {}: {}", status.as_str(), check_id, message)
    }
}

#[cfg(test)]
mod tests {
    use crate::usb::{Check, Report, Status};

    use super::{render_json, render_text};

    #[test]
    fn json_renderer_is_deterministic_with_schema_version() {
        let report = Report {
            schema_version: "usb-report.v1".to_string(),
            overall: Status::Pass,
            checks: vec![Check {
                status: Status::Pass,
                check_id: "a.check".to_string(),
                message: "ok".to_string(),
                hint: None,
            }],
            events: vec![crate::usb::Event {
                kind: "spawn_command_line".to_string(),
                text: "st-util -p 4242".to_string(),
            }],
        };
        let first = render_json(&report).expect("json render");
        let second = render_json(&report).expect("json render");
        assert_eq!(first, second);
        assert_eq!(
            first,
            "{\"schema_version\":\"usb-report.v1\",\"overall\":\"PASS\",\"checks\":[{\"status\":\"PASS\",\"check_id\":\"a.check\",\"message\":\"ok\",\"hint\":null}],\"events\":[{\"kind\":\"spawn_command_line\",\"text\":\"st-util -p 4242\"}]}"
        );
    }

    #[test]
    fn text_renderer_is_deterministic() {
        let report = Report {
            schema_version: "usb-report.v1".to_string(),
            overall: Status::Warn,
            checks: vec![
                Check {
                    status: Status::Pass,
                    check_id: "a.check".to_string(),
                    message: "ok".to_string(),
                    hint: None,
                },
                Check {
                    status: Status::Warn,
                    check_id: "b.check".to_string(),
                    message: "warn".to_string(),
                    hint: Some("fix it".to_string()),
                },
            ],
            events: vec![crate::usb::Event {
                kind: "spawn_command_line".to_string(),
                text: "openocd -f interface/stlink.cfg".to_string(),
            }],
        };
        let first = render_text(&report, true);
        let second = render_text(&report, true);
        assert_eq!(first, second);
        assert_eq!(
            first,
            "PASS a.check: ok\nWARN b.check: warn (hint: fix it)\nTRACE spawn_command_line: openocd -f interface/stlink.cfg"
        );
    }
}
