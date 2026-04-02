// PR 3: Scheduling Tools
// File: rust/crates/runtime/src/scheduler.rs
//
// Session-scoped in-memory cron scheduler for CronCreate/CronDelete/CronList tools.
// Jobs live only in the current session and are gone when the process exits.

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

// --- Cron Job ---

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CronJob {
    pub id: String,
    pub cron: String,
    pub prompt: String,
    pub recurring: bool,
    pub created_at: u64,
    pub enabled: bool,
}

// --- Scheduler ---

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Default)]
pub struct SessionScheduler {
    jobs: BTreeMap<String, CronJob>,
}

impl SessionScheduler {
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new cron job. Returns the job ID.
    pub fn create(
        &mut self,
        cron: String,
        prompt: String,
        recurring: bool,
    ) -> Result<CronJob, String> {
        // Validate cron expression (basic 5-field check)
        validate_cron_expression(&cron)?;

        let id = format!("cron_{}", NEXT_ID.fetch_add(1, Ordering::Relaxed));
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let job = CronJob {
            id: id.clone(),
            cron,
            prompt,
            recurring,
            created_at: now,
            enabled: true,
        };

        self.jobs.insert(id, job.clone());
        Ok(job)
    }

    /// Delete a job by ID. Returns true if the job existed.
    pub fn delete(&mut self, id: &str) -> bool {
        self.jobs.remove(id).is_some()
    }

    /// List all jobs.
    pub fn list(&self) -> Vec<&CronJob> {
        self.jobs.values().collect()
    }

    /// Get a specific job by ID.
    pub fn get(&self, id: &str) -> Option<&CronJob> {
        self.jobs.get(id)
    }

    /// Check which jobs should fire at the given minute (unix timestamp).
    /// Returns prompts to enqueue.
    pub fn tick(&mut self, now_unix: u64) -> Vec<String> {
        let mut prompts = Vec::new();
        let mut to_disable = Vec::new();

        for (id, job) in &self.jobs {
            if !job.enabled {
                continue;
            }
            if cron_matches(&job.cron, now_unix) {
                prompts.push(job.prompt.clone());
                if !job.recurring {
                    to_disable.push(id.clone());
                }
            }
        }

        for id in to_disable {
            if let Some(job) = self.jobs.get_mut(&id) {
                job.enabled = false;
            }
        }

        prompts
    }
}

// --- Cron Expression Parsing ---

fn validate_cron_expression(cron: &str) -> Result<(), String> {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return Err(format!(
            "cron expression must have exactly 5 fields (minute hour dom month dow), got {}",
            fields.len()
        ));
    }

    let labels = ["minute", "hour", "day-of-month", "month", "day-of-week"];
    let max_values = [59, 23, 31, 12, 7];

    for (i, field) in fields.iter().enumerate() {
        validate_cron_field(field, labels[i], max_values[i])?;
    }

    Ok(())
}

fn validate_cron_field(field: &str, label: &str, max: u32) -> Result<(), String> {
    if field == "*" {
        return Ok(());
    }

    for part in field.split(',') {
        let part = part.trim();
        if part.starts_with("*/") {
            let step: u32 = part[2..]
                .parse()
                .map_err(|_| format!("invalid step in {label}: {part}"))?;
            if step == 0 || step > max {
                return Err(format!("step out of range in {label}: {part}"));
            }
        } else if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() != 2 {
                return Err(format!("invalid range in {label}: {part}"));
            }
            let _low: u32 = bounds[0]
                .parse()
                .map_err(|_| format!("invalid range start in {label}: {part}"))?;
            let _high: u32 = bounds[1]
                .parse()
                .map_err(|_| format!("invalid range end in {label}: {part}"))?;
        } else {
            let _val: u32 = part
                .parse()
                .map_err(|_| format!("invalid value in {label}: {part}"))?;
        }
    }

    Ok(())
}

/// Check if a cron expression matches a given unix timestamp.
fn cron_matches(cron: &str, unix_ts: u64) -> bool {
    let fields: Vec<&str> = cron.split_whitespace().collect();
    if fields.len() != 5 {
        return false;
    }

    // Convert unix timestamp to components (UTC)
    let secs = unix_ts;
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let minute = (time_of_day / 60) % 60;
    let hour = time_of_day / 3600;

    // Calculate date from days since epoch (1970-01-01)
    let (year, month, dom) = days_to_ymd(days_since_epoch);
    let dow = ((days_since_epoch + 4) % 7) as u32; // 0=Sunday

    field_matches(fields[0], minute as u32)
        && field_matches(fields[1], hour as u32)
        && field_matches(fields[2], dom)
        && field_matches(fields[3], month)
        && field_matches(fields[4], dow)
}

fn field_matches(field: &str, value: u32) -> bool {
    if field == "*" {
        return true;
    }

    for part in field.split(',') {
        let part = part.trim();
        if part.starts_with("*/") {
            if let Ok(step) = part[2..].parse::<u32>() {
                if step > 0 && value % step == 0 {
                    return true;
                }
            }
        } else if part.contains('-') {
            let bounds: Vec<&str> = part.split('-').collect();
            if bounds.len() == 2 {
                if let (Ok(low), Ok(high)) = (bounds[0].parse::<u32>(), bounds[1].parse::<u32>()) {
                    if value >= low && value <= high {
                        return true;
                    }
                }
            }
        } else if let Ok(exact) = part.parse::<u32>() {
            if exact == value {
                return true;
            }
        }
    }

    false
}

fn days_to_ymd(mut days: u64) -> (u32, u32, u32) {
    // Simple Gregorian calendar conversion from days since 1970-01-01
    let mut year = 1970u32;
    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let month_days = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1u32;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }

    (year, month, days as u32 + 1)
}

fn is_leap_year(year: u32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

// Tool specs are defined in tools/src/lib.rs mvp_tool_specs() to avoid cross-crate dependency.

// --- Execution Handlers ---

#[derive(Debug, Deserialize)]
pub struct CronCreateInput {
    pub cron: String,
    pub prompt: String,
    #[serde(default = "default_recurring")]
    pub recurring: bool,
}

fn default_recurring() -> bool {
    true
}

#[derive(Debug, Deserialize)]
pub struct CronDeleteInput {
    pub id: String,
}

pub fn run_cron_create(
    input: CronCreateInput,
    scheduler: &mut SessionScheduler,
) -> Result<String, String> {
    let job = scheduler.create(input.cron, input.prompt, input.recurring)?;
    serde_json::to_string_pretty(&json!({
        "id": job.id,
        "cron": job.cron,
        "recurring": job.recurring,
        "status": "created"
    }))
    .map_err(|e| e.to_string())
}

pub fn run_cron_delete(
    input: CronDeleteInput,
    scheduler: &mut SessionScheduler,
) -> Result<String, String> {
    if scheduler.delete(&input.id) {
        Ok(format!("Job {} deleted.", input.id))
    } else {
        Err(format!("Job {} not found.", input.id))
    }
}

pub fn run_cron_list(scheduler: &SessionScheduler) -> Result<String, String> {
    let jobs: Vec<_> = scheduler.list().iter().map(|j| {
        json!({
            "id": j.id,
            "cron": j.cron,
            "prompt": j.prompt,
            "recurring": j.recurring,
            "enabled": j.enabled,
        })
    }).collect();
    serde_json::to_string_pretty(&jobs).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_and_list_jobs() {
        let mut scheduler = SessionScheduler::new();
        let job = scheduler.create(
            "0 9 * * *".to_string(),
            "daily standup".to_string(),
            true,
        ).unwrap();
        assert!(job.id.starts_with("cron_"));
        assert_eq!(scheduler.list().len(), 1);
    }

    #[test]
    fn delete_job() {
        let mut scheduler = SessionScheduler::new();
        let job = scheduler.create("*/5 * * * *".to_string(), "check".to_string(), true).unwrap();
        assert!(scheduler.delete(&job.id));
        assert_eq!(scheduler.list().len(), 0);
        assert!(!scheduler.delete("nonexistent"));
    }

    #[test]
    fn invalid_cron_rejected() {
        let mut scheduler = SessionScheduler::new();
        assert!(scheduler.create("bad".to_string(), "test".to_string(), true).is_err());
        assert!(scheduler.create("* * *".to_string(), "test".to_string(), true).is_err());
    }

    #[test]
    fn one_shot_disables_after_fire() {
        let mut scheduler = SessionScheduler::new();
        // Create a one-shot job that matches minute=0, hour=0 (midnight UTC)
        scheduler.create("0 0 * * *".to_string(), "once".to_string(), false).unwrap();

        // Tick at midnight UTC epoch day 1 (86400)
        let prompts = scheduler.tick(86400);
        assert_eq!(prompts.len(), 1);
        assert_eq!(prompts[0], "once");

        // Should not fire again
        let prompts = scheduler.tick(86400 + 86400); // next day midnight
        assert_eq!(prompts.len(), 0);
    }

    #[test]
    fn cron_field_matching() {
        assert!(field_matches("*", 5));
        assert!(field_matches("5", 5));
        assert!(!field_matches("5", 6));
        assert!(field_matches("1-5", 3));
        assert!(!field_matches("1-5", 6));
        assert!(field_matches("*/5", 10));
        assert!(!field_matches("*/5", 11));
        assert!(field_matches("1,3,5", 3));
    }

    #[test]
    fn validate_cron_expressions() {
        assert!(validate_cron_expression("0 9 * * *").is_ok());
        assert!(validate_cron_expression("*/5 * * * *").is_ok());
        assert!(validate_cron_expression("0 9 * * 1-5").is_ok());
        assert!(validate_cron_expression("bad").is_err());
        assert!(validate_cron_expression("0 9 *").is_err());
    }
}
