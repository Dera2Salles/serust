use crate::database::domain::DbAccessLog;
use crate::database::entities::{prelude::*, access_log};
use crate::database::interfaces::IAccessLogRepository;
use crate::database::Database;
use anyhow::Result;
use async_trait::async_trait;
use sea_orm::*;

#[derive(Clone)]
pub struct AccessLogRepository {
    db: Database,
}

impl AccessLogRepository {
    pub fn new(db: Database) -> Self {
        Self { db }
    }
}

#[async_trait]
impl IAccessLogRepository for AccessLogRepository {
    async fn create(&self, log: &DbAccessLog) -> Result<()> {
        let active_model = access_log::ActiveModel {
            file_id: Set(log.file_id.to_string()),
            accessed_by: Set(log.accessed_by.map(|u| u.to_string())),
            share_link_id: Set(log.share_link_id.map(|u| u.to_string())),
            grant_id: Set(log.grant_id.map(|u| u.to_string())),
            action: Set(Some(log.action.clone())),
            accessed_at: Set(log.accessed_at.into()),
            ip_address: Set(log.ip_address.clone()),
            user_agent: Set(log.user_agent.clone()),
            bytes_transferred: Set(log.bytes_transferred),
            ..Default::default()
        };

        AccessLog::insert(active_model).exec(&self.db.connection).await?;
        Ok(())
    }
}
