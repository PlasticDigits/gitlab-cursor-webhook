// SPDX-License-Identifier: AGPL-3.0-or-later

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use clap::{Parser, Subcommand};
use gch_core::{
    db::Database,
    prompt::{render_prompt, PromptContext},
    select_issue_agent, should_forward, should_forward_issue, tag::WebhookTag, IssueAgent,
};

#[derive(Parser)]
#[command(name = "gchconfig", about = "GitLab Cloud Host configuration CLI")]
struct Cli {
    #[arg(long, env = "GCH_DB_PATH", default_value = "/var/lib/gch/gch.db")]
    db: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Manage GitLab projects
    Project {
        #[command(subcommand)]
        action: ProjectCommands,
    },
    /// Manage agent tags (security, verify, implement)
    Tag {
        #[command(subcommand)]
        action: TagCommands,
    },
    /// Manage prompt templates per tag
    Prompt {
        #[command(subcommand)]
        action: PromptCommands,
    },
    /// Set global settings
    Setting {
        /// Setting key (controller_url, firewall_id, job_timeout_secs, etc.)
        key: String,
        /// Setting value
        value: String,
    },
    /// Validate configuration and external dependencies
    Doctor,
    /// Simulate webhook processing without provisioning a VM
    DryRun {
        /// Path to GitLab webhook JSON fixture
        #[arg(long)]
        fixture: PathBuf,
        /// Comma-separated allowed users override
        #[arg(long)]
        allowed_users: Option<String>,
    },
    /// Import legacy PROJECT_WEBHOOKS_* env entries into SQLite (paths only)
    ImportEnv,
}

#[derive(Subcommand)]
enum ProjectCommands {
    Add {
        #[arg(long)]
        gitlab: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long, default_value = "/home/agent/workspace")]
        workspace: String,
    },
    List,
    Remove {
        #[arg(long)]
        gitlab: String,
    },
}

#[derive(Subcommand)]
enum TagCommands {
    Add {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        snapshot: String,
        #[arg(long, default_value = "cx33")]
        server_type: String,
        #[arg(long, default_value = "nbg1")]
        location: String,
        #[arg(long, default_value = "composer-2.5")]
        model: String,
    },
    List {
        #[arg(long)]
        project: Option<String>,
    },
    SetSnapshot {
        #[arg(long)]
        project: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        snapshot: String,
    },
}

#[derive(Subcommand)]
enum PromptCommands {
    Set {
        #[arg(long)]
        project: String,
        #[arg(long)]
        tag: String,
        #[arg(long)]
        file: PathBuf,
    },
    Show {
        #[arg(long)]
        project: String,
        #[arg(long)]
        tag: String,
    },
}

fn main() {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    if let Err(e) = run(cli) {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

fn run(cli: Cli) -> Result<(), Box<dyn std::error::Error>> {
    let db = Database::open(&cli.db)?;

    match cli.command {
        Commands::Project { action } => match action {
            ProjectCommands::Add {
                gitlab,
                name,
                workspace,
            } => {
                let name = name.unwrap_or_else(|| {
                    gitlab.rsplit('/').next().unwrap_or(&gitlab).to_string()
                });
                let id = db.add_project(&gitlab, &name, &workspace)?;
                println!("project added id={id} gitlab_path={gitlab}");
            }
            ProjectCommands::List => {
                for p in db.list_projects()? {
                    let status = if p.enabled { "enabled" } else { "disabled" };
                    println!(
                        "{} ({}) workspace={} [{}]",
                        p.gitlab_path, p.name, p.workspace_path, status
                    );
                }
            }
            ProjectCommands::Remove { gitlab } => {
                if db.remove_project(&gitlab)? {
                    println!("removed project {gitlab}");
                } else {
                    println!("project not found: {gitlab}");
                }
            }
        },
        Commands::Tag { action } => match action {
            TagCommands::Add {
                project,
                name,
                snapshot,
                server_type,
                location,
                model,
            } => {
                let id = db.add_tag(
                    &project,
                    &name,
                    &snapshot,
                    Some(&server_type),
                    Some(&location),
                    Some(&model),
                )?;
                println!("tag added id={id} project={project} name={name} snapshot={snapshot}");
            }
            TagCommands::List { project } => {
                for (p, t) in db.list_tags(project.as_deref())? {
                    println!(
                        "{}:{} snapshot={} type={} loc={} model={}",
                        p.gitlab_path, t.name, t.hetzner_snapshot_id, t.server_type, t.hetzner_location, t.model
                    );
                }
            }
            TagCommands::SetSnapshot {
                project,
                name,
                snapshot,
            } => {
                db.set_tag_snapshot(&project, &name, &snapshot)?;
                println!("updated snapshot for {project}:{name} -> {snapshot}");
            }
        },
        Commands::Prompt { action } => match action {
            PromptCommands::Set { project, tag, file } => {
                let template = fs::read_to_string(&file)?;
                db.set_prompt(&project, &tag, &template)?;
                println!("prompt set for {project}:{tag} from {}", file.display());
            }
            PromptCommands::Show { project, tag } => {
                let template = db.get_prompt(&project, &tag)?;
                print!("{template}");
            }
        },
        Commands::Setting { key, value } => {
            db.set_setting(&key, &value)?;
            println!("setting {key}={value}");
        }
        Commands::Doctor => run_doctor(&db)?,
        Commands::DryRun {
            fixture,
            allowed_users,
        } => run_dry_run(&db, &fixture, allowed_users.as_deref())?,
        Commands::ImportEnv => run_import_env(&db)?,
    }

    Ok(())
}

fn run_doctor(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let mut ok = true;

    println!("== gchconfig doctor ==");

    let projects = db.list_projects()?;
    if projects.is_empty() {
        println!("[WARN] no projects configured");
        ok = false;
    } else {
        println!("[OK] {} project(s)", projects.len());
    }

    let tags = db.list_tags(None)?;
    if tags.is_empty() {
        println!("[WARN] no tags configured");
        ok = false;
    } else {
        for (p, t) in &tags {
            let prompt_ok = db.get_prompt(&p.name, &t.name).is_ok()
                || db.get_prompt(&p.gitlab_path, &t.name).is_ok();
            if !prompt_ok {
                println!("[WARN] missing prompt for {}:{}", p.gitlab_path, t.name);
                ok = false;
            }
            if t.hetzner_snapshot_id.is_empty() {
                println!("[WARN] empty snapshot for {}:{}", p.gitlab_path, t.name);
                ok = false;
            }
        }
        println!("[OK] {} tag(s)", tags.len());
    }

    let settings = db.get_settings()?;
    if settings.controller_url.is_empty() {
        println!("[WARN] controller_url not set (settings or GCH_CONTROLLER_URL)");
        ok = false;
    } else {
        println!("[OK] controller_url={}", settings.controller_url);
    }

    if settings.firewall_id.is_empty() && std::env::var("GCH_FIREWALL_ID").is_err() {
        println!("[WARN] firewall_id not set");
        ok = false;
    } else {
        println!("[OK] firewall configured");
    }

    if std::env::var("HCLOUD_TOKEN").is_err() {
        println!("[WARN] HCLOUD_TOKEN not set in environment");
        ok = false;
    } else {
        println!("[OK] HCLOUD_TOKEN set");
    }

    for bin in ["terraform", "hcloud"] {
        match Command::new(bin).arg("version").output() {
            Ok(o) if o.status.success() => println!("[OK] {bin} available"),
            _ => {
                println!("[WARN] {bin} not found or failed");
                ok = false;
            }
        }
    }

    if ok {
        println!("\nAll checks passed.");
    } else {
        println!("\nSome checks failed.");
        std::process::exit(1);
    }

    Ok(())
}

fn run_dry_run(
    db: &Database,
    fixture: &PathBuf,
    allowed_users: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let body = fs::read_to_string(fixture)?;
    let allowed: std::collections::HashSet<String> = allowed_users
        .unwrap_or("plasticdigits,brouie")
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();

    let envelope: gch_core::WebhookEnvelope = serde_json::from_str(&body)?;
    println!("object_kind={}", envelope.object_kind);

    match envelope.object_kind.as_str() {
        "merge_request" => {
            let payload: gch_core::GitLabMrWebhook = serde_json::from_str(&body)?;
            should_forward(&payload, &allowed).map_err(|r| format!("filter: {}", r.as_str()))?;
            let tag = WebhookTag::Security;
            let resolved = db.resolve_tag(&payload.project, tag)?;
            let Some(resolved) = resolved else {
                println!("result=skipped reason=project_not_configured");
                return Ok(());
            };
            let ctx = mr_prompt_context(&payload);
            let prompt = render_prompt(&resolved.prompt_template, &ctx);
            print_resolution(&resolved.project.gitlab_path, tag, &prompt);
        }
        "issue" => {
            let payload: gch_core::GitLabIssueWebhook = serde_json::from_str(&body)?;
            let agents =
                should_forward_issue(&payload, &allowed).map_err(|r| format!("filter: {}", r.as_str()))?;
            let agent = select_issue_agent(&agents).expect("non-empty agents");
            let tag = WebhookTag::from_issue_agent(agent);
            let resolved = db.resolve_tag(&payload.project, tag)?;
            let Some(resolved) = resolved else {
                println!("result=skipped reason=project_not_configured");
                return Ok(());
            };
            let ctx = issue_prompt_context(&payload, agent);
            let prompt = render_prompt(&resolved.prompt_template, &ctx);
            print_resolution(&resolved.project.gitlab_path, tag, &prompt);
        }
        other => println!("result=skipped reason=unsupported_object_kind ({other})"),
    }

    Ok(())
}

fn print_resolution(project: &str, tag: WebhookTag, prompt: &str) {
    println!("result=would_provision");
    println!("project={project}");
    println!("tag={tag}");
    println!("--- prompt ---");
    println!("{prompt}");
    println!("--- end prompt ---");
}

fn mr_prompt_context(payload: &gch_core::GitLabMrWebhook) -> PromptContext {
    let mut ctx = PromptContext::default();
    ctx.insert("event_type", &payload.object_attributes.action);
    ctx.insert("username", &payload.user.username);
    ctx.insert("project_name", &payload.project.name);
    ctx.insert("web_url", &payload.object_attributes.url);
    ctx.insert(
        "description",
        payload.object_attributes.description.as_deref().unwrap_or(""),
    );
    ctx.insert("iid", payload.object_attributes.iid.to_string());
    ctx.insert("title", &payload.object_attributes.title);
    if let Some(c) = &payload.object_attributes.last_commit {
        ctx.insert("last_commit_id", &c.id);
        ctx.insert("last_commit_message", c.message.as_deref().unwrap_or(""));
    }
    ctx
}

fn issue_prompt_context(payload: &gch_core::GitLabIssueWebhook, agent: IssueAgent) -> PromptContext {
    let mut ctx = PromptContext::default();
    ctx.insert("event_type", &payload.object_attributes.action);
    ctx.insert("agent", agent.as_str());
    ctx.insert("username", &payload.user.username);
    ctx.insert("project_name", &payload.project.name);
    ctx.insert(
        "web_url",
        payload.object_attributes.url.as_deref().unwrap_or(""),
    );
    ctx.insert(
        "description",
        payload.object_attributes.description.as_deref().unwrap_or(""),
    );
    ctx.insert("iid", payload.object_attributes.iid.to_string());
    ctx.insert("title", &payload.object_attributes.title);
    let labels: Vec<&str> = payload.labels.iter().map(|l| l.title.as_str()).collect();
    ctx.insert("labels", labels.join(", "));
    ctx
}

fn run_import_env(db: &Database) -> Result<(), Box<dyn std::error::Error>> {
    let maps = [
        ("PROJECT_WEBHOOKS_SECURITY", "security"),
        ("PROJECT_WEBHOOKS_VERIFY", "verify"),
        ("PROJECT_WEBHOOKS_IMPLEMENT", "implement"),
    ];

    let mut imported = 0;
    for (var, _tag) in maps {
        let Ok(raw) = std::env::var(var) else { continue };
        for entry in raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
            let Some((gitlab_path, _)) = entry.split_once('=') else { continue };
            let gitlab_path = gitlab_path.trim();
            if db.find_project_by_gitlab_path(gitlab_path)?.is_none() {
                let name = gitlab_path.rsplit('/').next().unwrap_or(gitlab_path);
                db.add_project(gitlab_path, name, "/home/agent/workspace")?;
                println!("imported project {gitlab_path}");
                imported += 1;
            }
        }
    }

    if imported == 0 {
        println!("no new projects imported (set PROJECT_WEBHOOKS_* or projects already exist)");
    } else {
        println!("imported {imported} project(s); add tags and snapshots with gchconfig tag add");
    }

    Ok(())
}
