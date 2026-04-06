use crate::application::share_service::ShareService;
use crate::domain::user::User;
use crate::framework::{
    context::Context,
    handler::{Handler, HandlerResult},
};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::io::BufReader;
use tracing::error;

fn make_user(ctx: &Context) -> User {
    User { username: ctx.user().username.clone(), password_hash: String::new() }
}

fn parse_perm(s: &str) -> (bool, bool, bool) {
    let up = s.to_uppercase();
    let can_read = up.contains('R');
    let can_write = up.contains('W');
    let can_download = up.contains('D');
    (can_read, can_write, can_download)
}

fn parse_opt_u64(arg: &str, key: &str) -> Option<u64> {
    let prefix = format!("{}=", key);
    let v = arg.strip_prefix(&prefix)?;
    v.parse::<u64>().ok()
}

fn parse_opt_bool(arg: &str, key: &str) -> Option<bool> {
    let prefix = format!("{}=", key);
    let v = arg.strip_prefix(&prefix)?;
    match v {
        "1" | "true" | "TRUE" => Some(true),
        "0" | "false" | "FALSE" => Some(false),
        _ => None,
    }
}

pub struct ShareHandler {
    shares: Arc<ShareService>,
}

impl ShareHandler {
    pub fn new(shares: Arc<ShareService>) -> Self {
        Self { shares }
    }
}

#[async_trait]
impl Handler for ShareHandler {
    fn command(&self) -> &'static str { "SHARE" }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.len() < 3 {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let actor = make_user(ctx);
        let cwd = ctx.cwd.clone();
        let path = args[0];
        let to_user = args[1];
        let (can_read, can_write, can_download) = parse_perm(args[2]);

        let mut remaining_reads: Option<u64> = None;
        let mut remaining_writes: Option<u64> = None;
        let mut remaining_downloads: Option<u64> = None;
        let mut expires_at: Option<u64> = None;
        let mut can_reshare = false;

        for opt in &args[3..] {
            if let Some(v) = parse_opt_u64(opt, "reads") {
                remaining_reads = Some(v);
            } else if let Some(v) = parse_opt_u64(opt, "writes") {
                remaining_writes = Some(v);
            } else if let Some(v) = parse_opt_u64(opt, "downloads") {
                remaining_downloads = Some(v);
            } else if let Some(v) = parse_opt_u64(opt, "expires") {
                expires_at = Some(v);
            } else if let Some(v) = parse_opt_bool(opt, "reshare") {
                can_reshare = v;
            }
        }

        let (owner, owner_rel) = match ShareService::resolve_owner_path(&actor.username, &cwd, path) {
            Ok(v) => v,
            Err(_) => { ctx.error(550, "Requested action not taken."); return Ok(()); }
        };

        if actor.username != owner {
            if !self.shares.can_reshare(&actor.username, &owner, &owner_rel).await {
                ctx.error(550, "Requested action not taken.");
                return Ok(());
            }
        }

        match self
            .shares
            .grant(
                &owner,
                "/",
                &owner_rel,
                to_user,
                can_read,
                can_write,
                can_download,
                remaining_reads,
                remaining_writes,
                remaining_downloads,
                can_reshare,
                &actor.username,
                expires_at,
            )
            .await
        {
            Ok(_) => ctx.write_line("200 Share created."),
            Err(e) => {
                error!("SHARE: {}", e);
                ctx.error(550, "Requested action not taken.");
            }
        }

        Ok(())
    }
}

pub struct UnshareHandler {
    shares: Arc<ShareService>,
}

impl UnshareHandler {
    pub fn new(shares: Arc<ShareService>) -> Self {
        Self { shares }
    }
}

#[async_trait]
impl Handler for UnshareHandler {
    fn command(&self) -> &'static str { "UNSHARE" }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        if args.len() < 2 {
            ctx.error(501, "Syntax error in parameters or arguments.");
            return Ok(());
        }

        let actor = make_user(ctx);
        let cwd = ctx.cwd.clone();
        let path = args[0];
        let to_user = args[1];

        let (owner, owner_rel) = match ShareService::resolve_owner_path(&actor.username, &cwd, path) {
            Ok(v) => v,
            Err(_) => { ctx.error(550, "Requested action not taken."); return Ok(()); }
        };
        if actor.username != owner {
            ctx.error(550, "Requested action not taken.");
            return Ok(());
        }

        match self.shares.revoke(&owner, "/", &owner_rel, to_user).await {
            Ok(_) => ctx.write_line("200 Share revoked."),
            Err(_) => ctx.error(550, "Requested action not taken."),
        }

        Ok(())
    }
}

pub struct SharesHandler {
    shares: Arc<ShareService>,
}

impl SharesHandler {
    pub fn new(shares: Arc<ShareService>) -> Self {
        Self { shares }
    }
}

#[async_trait]
impl Handler for SharesHandler {
    fn command(&self) -> &'static str { "SHARES" }
    async fn handle(
        &self,
        ctx: &mut Context,
        args: &[&str],
        _: &mut BufReader<tokio::net::tcp::ReadHalf<'_>>,
        _writer: &mut tokio::net::tcp::WriteHalf<'_>,
    ) -> HandlerResult {
        let actor = make_user(ctx);
        let mode = args.first().map(|s| s.to_uppercase()).unwrap_or_else(|| "IN".to_string());

        let rows = if mode == "OUT" {
            self.shares.list_outgoing(&actor.username).await
        } else {
            self.shares.list_incoming(&actor.username).await
        };

        ctx.write_line("211-Share list:");
        for g in rows {
            let perm = format!("{}{}{}", 
                if g.can_read { "R" } else { "-" },
                if g.can_write { "W" } else { "-" },
                if g.can_download { "D" } else { "-" }
            );
            let rr = g.remaining_reads.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            let rw = g.remaining_writes.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            let rd = g.remaining_downloads.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            let exp = g.expires_at.map(|v| v.to_string()).unwrap_or_else(|| "-".to_string());
            ctx.write_line(&format!(
                " owner={} grantee={} path=/{} perm={} reads={} writes={} downloads={} reshare={} expires={} by={}",
                g.owner,
                g.grantee,
                g.path,
                perm,
                rr,
                rw,
                rd,
                if g.can_reshare { 1 } else { 0 },
                exp,
                g.granted_by
            ));
        }
        ctx.write_line("211 End");
        Ok(())
    }
}

