use anyhow::{Context, Result};

#[cfg(test)]
#[path = "notices/tests.rs"]
mod tests;

pub(super) fn print_recipient_reminders(result: &serde_json::Value) -> Result<()> {
    for reminder in recipient_reminders(result)? {
        println!("{reminder}");
    }
    Ok(())
}

fn recipient_reminders(result: &serde_json::Value) -> Result<Vec<&str>> {
    result
        .get("recipient_reminders")
        .and_then(serde_json::Value::as_array)
        .context("daemon response missing recipient_reminders")?
        .iter()
        .map(|value| {
            value
                .as_str()
                .context("daemon returned a non-string recipient reminder")
        })
        .collect()
}
