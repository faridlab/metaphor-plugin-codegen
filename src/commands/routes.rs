//! `routes` command — list all HTTP routes defined in the current project.
//!
//! Scans Rust source files under a given path (default `src/`) for two patterns:
//!
//! 1. Raw Axum routes: `.route("/path", method(handler))` and chained forms.
//! 2. Backbone CRUD handlers: `BackboneCrudHandler::<_>::routes(_, "/path")`,
//!    which expands into the standard 15-endpoint CRUD surface.
//!
//! Nested routers (`Router::nest("/prefix", ...)`) are not yet followed —
//! this printer reports routes at their literal mount path.

use anyhow::Result;
use clap::{Args, ValueEnum};
use colored::*;
use once_cell::sync::Lazy;
use regex::Regex;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Args, Clone, Debug)]
pub struct RoutesArgs {
    /// Directory to scan (default: src/)
    #[arg(long, default_value = "src")]
    pub path: PathBuf,

    /// Output format
    #[arg(long, value_enum, default_value_t = RoutesFormat::Table)]
    pub format: RoutesFormat,

    /// Only show routes whose path contains this substring
    #[arg(long)]
    pub filter: Option<String>,
}

#[derive(Copy, Clone, Debug, ValueEnum, PartialEq, Eq)]
pub enum RoutesFormat {
    Table,
    List,
    Json,
    Markdown,
}

#[derive(Debug, Clone)]
struct RouteEntry {
    method: String,
    path: String,
    source: String,
    handler: Option<String>,
}

/// Endpoints produced by `BackboneCrudHandler::<_>::routes(_, "/base")`.
/// Kept in sync with backbone-core — update here when CRUD surface changes.
const BACKBONE_CRUD_ENDPOINTS: &[(&str, &str, &str)] = &[
    ("GET",    "",                 "list"),
    ("POST",   "",                 "create"),
    ("GET",    "/:id",             "get by id"),
    ("PUT",    "/:id",             "full update"),
    ("PATCH",  "/:id",             "partial update"),
    ("DELETE", "/:id",             "soft delete"),
    ("POST",   "/bulk",            "bulk create"),
    ("POST",   "/upsert",          "upsert"),
    ("GET",    "/trash",           "list deleted"),
    ("POST",   "/:id/restore",     "restore"),
    ("DELETE", "/empty",           "empty trash"),
    ("GET",    "/:id/deleted",     "get deleted by id"),
    ("DELETE", "/trash/:id",       "permanent delete"),
    ("GET",    "/count",           "count active"),
    ("GET",    "/trash/count",     "count deleted"),
];

static RE_ROUTE: Lazy<Regex> = Lazy::new(|| {
    // .route("/path", method(handler))
    // .route("/path", method(h1).method(h2))
    Regex::new(r#"\.route\(\s*"([^"]+)"\s*,\s*([^)]*?\([^)]*\)(?:\.[a-z]+\([^)]*\))*)\s*\)"#)
        .expect("RE_ROUTE is valid")
});

static RE_METHOD_CALL: Lazy<Regex> = Lazy::new(|| {
    // Matches `method(handler_name)` within a route expression
    Regex::new(r#"(get|post|put|patch|delete|head|options|trace)\(\s*([A-Za-z_][A-Za-z0-9_:]*)"#)
        .expect("RE_METHOD_CALL is valid")
});

static RE_BACKBONE_CRUD: Lazy<Regex> = Lazy::new(|| {
    // BackboneCrudHandler::<...>::routes(service, "/base")
    // Also catches simpler BackboneCrudHandler::routes(service, "/base")
    Regex::new(r#"BackboneCrudHandler(?:::<[^>]*>)?::routes\s*\([^,]*,\s*"([^"]+)"\s*\)"#)
        .expect("RE_BACKBONE_CRUD is valid")
});

pub async fn handle_command(args: &RoutesArgs) -> Result<()> {
    let root = &args.path;
    if !root.exists() {
        anyhow::bail!(
            "Path {} does not exist. Run from a project root or pass --path.",
            root.display()
        );
    }

    let mut routes = Vec::new();
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = match entry {
            Ok(e) => e,
            Err(_) => continue,
        };
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = match std::fs::read_to_string(entry.path()) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = entry.path().strip_prefix(root).unwrap_or(entry.path());
        let source = rel.display().to_string();
        collect_routes(&content, &source, &mut routes);
    }

    if let Some(filter) = &args.filter {
        routes.retain(|r| r.path.contains(filter));
    }

    // Stable sort: path, then method
    routes.sort_by(|a, b| a.path.cmp(&b.path).then(a.method.cmp(&b.method)));

    if routes.is_empty() {
        println!("{} No routes found under {}", "⚠️".yellow(), root.display());
        return Ok(());
    }

    match args.format {
        RoutesFormat::Table => print_table(&routes),
        RoutesFormat::List => print_list(&routes),
        RoutesFormat::Markdown => print_markdown(&routes),
        RoutesFormat::Json => print_json(&routes)?,
    }

    Ok(())
}

fn collect_routes(content: &str, source: &str, out: &mut Vec<RouteEntry>) {
    // Pattern 1: raw Axum routes
    for caps in RE_ROUTE.captures_iter(content) {
        let path = caps[1].to_string();
        let expr = &caps[2];
        let mut methods = Vec::new();
        for m in RE_METHOD_CALL.captures_iter(expr) {
            methods.push((m[1].to_uppercase(), m[2].to_string()));
        }
        if methods.is_empty() {
            out.push(RouteEntry {
                method: "ANY".to_string(),
                path: path.clone(),
                source: source.to_string(),
                handler: None,
            });
        } else {
            for (method, handler) in methods {
                out.push(RouteEntry {
                    method,
                    path: path.clone(),
                    source: source.to_string(),
                    handler: Some(handler),
                });
            }
        }
    }

    // Pattern 2: BackboneCrudHandler — expand to the standard CRUD surface
    for caps in RE_BACKBONE_CRUD.captures_iter(content) {
        let base = caps[1].trim_end_matches('/');
        for (method, suffix, desc) in BACKBONE_CRUD_ENDPOINTS {
            out.push(RouteEntry {
                method: (*method).to_string(),
                path: format!("{}{}", base, suffix),
                source: source.to_string(),
                handler: Some((*desc).to_string()),
            });
        }
    }
}

fn print_table(routes: &[RouteEntry]) {
    let method_w = routes.iter().map(|r| r.method.len()).max().unwrap_or(6).max(6);
    let path_w = routes.iter().map(|r| r.path.len()).max().unwrap_or(4).max(4);
    let handler_w = routes
        .iter()
        .map(|r| r.handler.as_deref().unwrap_or("").len())
        .max()
        .unwrap_or(0)
        .max(7);

    println!(
        "{:<method_w$}  {:<path_w$}  {:<handler_w$}  {}",
        "METHOD".bold(),
        "PATH".bold(),
        "HANDLER".bold(),
        "SOURCE".bold(),
        method_w = method_w,
        path_w = path_w,
        handler_w = handler_w,
    );
    println!(
        "{}  {}  {}  {}",
        "-".repeat(method_w),
        "-".repeat(path_w),
        "-".repeat(handler_w),
        "-".repeat(6),
    );
    for r in routes {
        let method_colored = match r.method.as_str() {
            "GET" => r.method.green(),
            "POST" => r.method.yellow(),
            "PUT" | "PATCH" => r.method.blue(),
            "DELETE" => r.method.red(),
            _ => r.method.normal(),
        };
        println!(
            "{:<method_w$}  {:<path_w$}  {:<handler_w$}  {}",
            method_colored,
            r.path,
            r.handler.as_deref().unwrap_or(""),
            r.source.dimmed(),
            method_w = method_w,
            path_w = path_w,
            handler_w = handler_w,
        );
    }
    println!();
    println!("{} {} routes", "Total:".bold(), routes.len().to_string().bright_cyan());
}

fn print_list(routes: &[RouteEntry]) {
    for r in routes {
        println!("{:<7} {}", r.method, r.path);
    }
}

fn print_markdown(routes: &[RouteEntry]) {
    println!("| Method | Path | Handler | Source |");
    println!("|--------|------|---------|--------|");
    for r in routes {
        println!(
            "| {} | `{}` | {} | `{}` |",
            r.method,
            r.path,
            r.handler.as_deref().unwrap_or(""),
            r.source
        );
    }
}

fn print_json(routes: &[RouteEntry]) -> Result<()> {
    let payload: Vec<_> = routes
        .iter()
        .map(|r| {
            serde_json::json!({
                "method": r.method,
                "path": r.path,
                "handler": r.handler,
                "source": r.source,
            })
        })
        .collect();
    println!("{}", serde_json::to_string_pretty(&payload)?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_raw_routes() {
        let src = r#"
            Router::new()
                .route("/health", get(health))
                .route("/users/:id", get(get_user).delete(delete_user))
        "#;
        let mut out = Vec::new();
        collect_routes(src, "test.rs", &mut out);
        assert!(out.iter().any(|r| r.method == "GET" && r.path == "/health"));
        assert!(out.iter().any(|r| r.method == "GET" && r.path == "/users/:id"));
        assert!(out.iter().any(|r| r.method == "DELETE" && r.path == "/users/:id"));
    }

    #[test]
    fn expands_backbone_crud() {
        let src = r#"
            BackboneCrudHandler::<AccountService, Account, CreateAccountDto, UpdateAccountDto, AccountResponseDto>::routes(
                service,
                "/accounts",
            )
        "#;
        let mut out = Vec::new();
        collect_routes(src, "account.rs", &mut out);
        assert_eq!(out.len(), BACKBONE_CRUD_ENDPOINTS.len());
        assert!(out.iter().any(|r| r.method == "GET" && r.path == "/accounts"));
        assert!(out.iter().any(|r| r.method == "DELETE" && r.path == "/accounts/:id"));
    }
}
