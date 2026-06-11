// SPDX-License-Identifier: AGPL-3.0-or-later

use std::path::{Path, PathBuf};

use gch_core::{
    db::Database,
    job_api::{DiskJobWorkspace, JobListResponse, JobSummary},
    list_disk_workspaces,
};
use reqwest::blocking::Client;
use uuid::Uuid;

pub fn run_jobs_list(
    db: &Database,
    jobs_dir: &Path,
    active: bool,
    disk_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    if disk_only {
        print_disk_workspaces(jobs_dir);
        return Ok(());
    }

    match fetch_jobs_from_controller(db, active) {
        Ok(mut jobs) => {
            enrich_server_ips(&mut jobs, jobs_dir);
            if jobs.is_empty() {
                println!("no jobs in controller memory");
            } else {
                for job in &jobs {
                    print_job_summary(job);
                }
            }
            print_orphaned_disk_workspaces(jobs_dir, &jobs);
            Ok(())
        }
        Err(e) => {
            eprintln!("controller: {e}");
            eprintln!("falling back to on-disk terraform workspaces under {}", jobs_dir.display());
            print_disk_workspaces(jobs_dir);
            Ok(())
        }
    }
}

pub fn run_jobs_show(
    db: &Database,
    jobs_dir: &Path,
    job_id: &str,
    disk_only: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    Uuid::parse_str(job_id).map_err(|_| "invalid job id (expected UUID)")?;

    if disk_only {
        return show_disk_job(jobs_dir, job_id);
    }

    match fetch_job_from_controller(db, job_id) {
        Ok(mut job) => {
            enrich_server_ip(&mut job, jobs_dir);
            print_job_detail(&job);
            Ok(())
        }
        Err(e) => {
            eprintln!("controller: {e}");
            eprintln!("falling back to on-disk workspace");
            show_disk_job(jobs_dir, job_id)
        }
    }
}

fn controller_url(db: &Database) -> Result<String, String> {
    let settings = db.get_settings().map_err(|e| e.to_string())?;
    if settings.controller_url.is_empty() {
        return Err("controller_url not set (gchconfig setting controller_url ...)".into());
    }
    Ok(settings.controller_url.trim_end_matches('/').to_string())
}

fn admin_token() -> Result<String, String> {
    match std::env::var("GCH_ADMIN_TOKEN") {
        Ok(t) if !t.trim().is_empty() => Ok(t.trim().to_string()),
        _ => Err("GCH_ADMIN_TOKEN not set in environment".into()),
    }
}

fn fetch_jobs_from_controller(db: &Database, active: bool) -> Result<Vec<JobSummary>, String> {
    let base = controller_url(db)?;
    let token = admin_token()?;
    let url = if active {
        format!("{base}/api/admin/jobs?active=true")
    } else {
        format!("{base}/api/admin/jobs")
    };

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
        return Err("admin API disabled on controller (set GCH_ADMIN_TOKEN)".into());
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    let body: JobListResponse = response.json().map_err(|e| format!("invalid JSON: {e}"))?;
    Ok(body.jobs)
}

fn fetch_job_from_controller(db: &Database, job_id: &str) -> Result<JobSummary, String> {
    let base = controller_url(db)?;
    let token = admin_token()?;
    let url = format!("{base}/api/admin/jobs/{job_id}");

    let client = Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| e.to_string())?;

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {token}"))
        .send()
        .map_err(|e| format!("request failed: {e}"))?;

    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Err(format!("job not found in controller memory: {job_id}"));
    }
    if response.status() == reqwest::StatusCode::SERVICE_UNAVAILABLE {
        return Err("admin API disabled on controller (set GCH_ADMIN_TOKEN)".into());
    }
    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status()));
    }

    response.json().map_err(|e| format!("invalid JSON: {e}"))
}

fn enrich_server_ips(jobs: &mut [JobSummary], jobs_dir: &Path) {
    for job in jobs.iter_mut() {
        enrich_server_ip(job, jobs_dir);
    }
}

fn enrich_server_ip(job: &mut JobSummary, jobs_dir: &Path) {
    if job.server_ipv4.is_some() {
        return;
    }
    if let Some(disk) = list_disk_workspaces(jobs_dir)
        .into_iter()
        .find(|w| w.job_id == job.job_id)
    {
        job.server_ipv4 = disk.server_ipv4;
    }
}

fn print_job_summary(job: &JobSummary) {
    let heartbeat = job
        .last_heartbeat_secs_ago
        .map(|s| format!("heartbeat={s}s ago"))
        .unwrap_or_else(|| "heartbeat=none".to_string());
    let phase = job.phase.as_deref().unwrap_or("-");
    let ip = job.server_ipv4.as_deref().unwrap_or("-");
    println!(
        "{} status={} tag={} project={} iid={} phase={} ip={} age={}s {}",
        job.job_id, job.status, job.tag, job.project, job.iid, phase, ip, job.age_secs, heartbeat
    );
    if let Some(msg) = &job.status_message {
        println!("  message: {msg}");
    }
}

fn print_job_detail(job: &JobSummary) {
    println!("job_id: {}", job.job_id);
    println!("status: {}", job.status);
    println!("tag: {}", job.tag);
    println!("project: {}", job.project);
    println!("iid: {}", job.iid);
    println!("object_kind: {}", job.object_kind);
    println!("model: {}", job.model);
    println!("created_at: {}", job.created_at);
    if let Some(completed) = &job.completed_at {
        println!("completed_at: {completed}");
    }
    if let Some(phase) = &job.phase {
        println!("phase: {phase}");
    }
    if let Some(msg) = &job.status_message {
        println!("status_message: {msg}");
    }
    if let Some(secs) = job.last_heartbeat_secs_ago {
        println!("last_heartbeat_secs_ago: {secs}");
    }
    println!("age_secs: {}", job.age_secs);
    if let Some(ip) = &job.server_ipv4 {
        println!("server_ipv4: {ip}");
    }
    if let Some(ws) = &job.workspace_path {
        println!("workspace: {ws}");
    }
    if let Some(git_ref) = &job.git_ref {
        println!("git_ref: {git_ref}");
    }
}

fn print_disk_workspaces(jobs_dir: &Path) {
    let workspaces = list_disk_workspaces(jobs_dir);
    if workspaces.is_empty() {
        println!("no terraform workspaces under {}", jobs_dir.display());
        return;
    }
    for ws in workspaces {
        print_disk_workspace(&ws);
    }
}

fn print_disk_workspace(ws: &DiskJobWorkspace) {
    let tag = ws.tag.as_deref().unwrap_or("-");
    let project = ws.project.as_deref().unwrap_or("-");
    let iid = ws.iid.as_deref().unwrap_or("-");
    let ip = ws.server_ipv4.as_deref().unwrap_or("-");
    println!(
        "{} [disk] tag={tag} project={project} iid={iid} ip={ip}",
        ws.job_id
    );
}

fn show_disk_job(jobs_dir: &Path, job_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let workspace = list_disk_workspaces(jobs_dir)
        .into_iter()
        .find(|w| w.job_id == job_id)
        .ok_or_else(|| format!("no terraform workspace for job {job_id}"))?;

    print_disk_workspace(&workspace);
    Ok(())
}

fn print_orphaned_disk_workspaces(jobs_dir: &Path, controller_jobs: &[JobSummary]) {
    let known: std::collections::HashSet<&str> =
        controller_jobs.iter().map(|j| j.job_id.as_str()).collect();

    let orphans: Vec<DiskJobWorkspace> = list_disk_workspaces(jobs_dir)
        .into_iter()
        .filter(|w| !known.contains(w.job_id.as_str()))
        .collect();

    if orphans.is_empty() {
        return;
    }

    println!();
    println!("orphaned disk workspaces (not in controller memory):");
    for ws in orphans {
        print_disk_workspace(&ws);
    }
}

pub fn default_jobs_dir() -> PathBuf {
    PathBuf::from(
        std::env::var("GCH_JOBS_DIR").unwrap_or_else(|_| "/var/lib/gch/jobs".to_string()),
    )
}
