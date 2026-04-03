use xtask::render::{render_json, render_text};
use xtask::usb::Report;

pub(crate) fn emit_report(report: &Report, json: bool, verbose: bool) -> Result<(), String> {
    if json {
        let rendered = render_json(report).map_err(|err| format!("json render failed: {err}"))?;
        println!("{rendered}");
    } else {
        let rendered = render_text(report, verbose);
        if !rendered.is_empty() {
            println!("{rendered}");
        }
    }
    Ok(())
}
