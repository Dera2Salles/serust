use crate::file::service::FileService;
use crate::framework::app::App;
use crate::server::handlers::auth::{PassHandler, UserHandler};
use crate::server::handlers::dir::{CdupHandler, CwdHandler, DirHandler};
use crate::server::handlers::info::{
    FeatHandler, MlstHandler, NoopHandler, QuitHandler, SystHandler, TypeHandler,
};
use crate::server::handlers::network::{PasvHandler, PortHandler};
use crate::server::handlers::share::{ShareHandler, SharesHandler, UnshareHandler};
use crate::server::handlers::transfer::{
    DeleHandler, ListDirHandler, MkdHandler, NlstHandler, RetrHandler, RmdHandler, RnfrHandler,
    RntoHandler, SizeHandler, StorHandler,
};
use crate::server::middlewares::{
    auth_middleware::AuthMiddleware, logging_middleware::LoggingMiddleware,
    rate_limit_middleware::RateLimitMiddleware,
};
use crate::share::service::ShareService;
use crate::user::service::AuthService;
use std::sync::Arc;
use std::time::Duration;

pub struct TcpServer {
    app: App,
}

impl TcpServer {
    pub fn new(
        auth_service: Arc<AuthService>,
        file_service: Arc<FileService>,
        share_service: Arc<ShareService>,
    ) -> Self {
        let app = App::new()
            .banner("220 tcp-framework FTP Server ready.")
            .max_connections(1024)
            .timeout(Duration::from_secs(300))
            .middleware(LoggingMiddleware)
            .middleware(RateLimitMiddleware::new(200, Duration::from_secs(60)))
            .middleware(AuthMiddleware)
            .route(UserHandler)
            .route(PassHandler::new(Arc::clone(&auth_service)))
            .route(SystHandler)
            .route(FeatHandler)
            .route(TypeHandler)
            .route(NoopHandler)
            .route(QuitHandler)
            .route(MlstHandler::new(Arc::clone(&file_service)))
            .route(PasvHandler)
            .route(PortHandler)
            .route(DirHandler)
            .route(CwdHandler::new(Arc::clone(&file_service)))
            .route(CdupHandler)
            .route(StorHandler::new(Arc::clone(&file_service)))
            .route(RetrHandler::new(Arc::clone(&file_service)))
            .route(ListDirHandler::new(Arc::clone(&file_service)))
            .route(NlstHandler::new(Arc::clone(&file_service)))
            .route(MkdHandler::new(Arc::clone(&file_service)))
            .route(RmdHandler::new(Arc::clone(&file_service)))
            .route(DeleHandler::new(Arc::clone(&file_service)))
            .route(RnfrHandler)
            .route(RntoHandler::new(Arc::clone(&file_service)))
            .route(SizeHandler::new(Arc::clone(&file_service)))
            .route(ShareHandler::new(Arc::clone(&share_service)))
            .route(UnshareHandler::new(Arc::clone(&share_service)))
            .route(SharesHandler::new(Arc::clone(&share_service)));

        Self { app }
    }

    pub async fn run(self, addr: &str) -> anyhow::Result<()> {
        self.app.run(addr).await
    }
}
