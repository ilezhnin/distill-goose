use anyhow::Result;
use futures::future::BoxFuture;
use std::path::PathBuf;

use crate::acp::{
    extension_configs_to_mcp_servers, AcpProvider, AcpProviderConfig, ACP_CURRENT_MODEL,
};
use crate::config::search_path::SearchPaths;
use crate::config::{Config, GooseMode};
use crate::providers::base::{
    current_working_dir, ProviderDef, ProviderDescriptor, ProviderMetadata,
};
use crate::providers::catalog::ProviderSetupMetadata;

pub(crate) const GROK_ACP_PROVIDER_NAME: &str = "grok-acp";
pub(crate) const GROK_ACP_BINARY: &str = "grok";
const GROK_ACP_DOC_URL: &str = "https://docs.x.ai/build/cli/reference";

pub struct GrokAcpProvider;

impl goose_providers::base::ProviderDescriptor for GrokAcpProvider {
    fn metadata() -> ProviderMetadata {
        ProviderMetadata::new(
            GROK_ACP_PROVIDER_NAME,
            "Grok CLI ACP",
            "Use goose with Grok Build through the xAI Grok CLI ACP agent.",
            ACP_CURRENT_MODEL,
            vec![],
            GROK_ACP_DOC_URL,
            vec![],
        )
        .with_setup_steps(vec![
            "Install the Grok CLI from xAI's installer.",
            "Authenticate with xAI: run `grok login` or set `XAI_API_KEY`.",
            "Verify ACP startup: `grok agent stdio`.",
        ])
        .with_setup(
            ProviderSetupMetadata::cli_agent(
                GROK_ACP_BINARY,
                &["grok-acp", "grok_cli", "grok-cli", "grok"],
            )
            .with_acp()
            .with_docs_url(GROK_ACP_DOC_URL)
            .with_capabilities(false, false, false),
        )
    }
}

impl ProviderDef for GrokAcpProvider {
    type Provider = AcpProvider;

    fn from_env(
        extensions: Vec<crate::config::ExtensionConfig>,
        tls_config: Option<crate::providers::api_client::TlsConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Self::from_env_with_working_dir(extensions, current_working_dir(), tls_config)
    }

    fn from_env_with_working_dir(
        extensions: Vec<crate::config::ExtensionConfig>,
        working_dir: PathBuf,
        _tls_config: Option<crate::providers::api_client::TlsConfig>,
    ) -> BoxFuture<'static, Result<AcpProvider>> {
        Box::pin(async move {
            let config = Config::global();
            let resolved_command = SearchPaths::builder().with_npm().resolve(GROK_ACP_BINARY)?;
            let goose_mode = config.get_goose_mode().unwrap_or(GooseMode::Auto);
            let mcp_servers = extension_configs_to_mcp_servers(&extensions);

            let provider_config = AcpProviderConfig {
                command: resolved_command,
                args: vec!["agent".to_string(), "stdio".to_string()],
                env: vec![],
                env_remove: vec![],
                work_dir: working_dir,
                mcp_servers,
                session_mode_id: None,
                session_config_options: vec![],
                model_config_option_id: None,
                mode_mapping: Default::default(),
                notification_callback: None,
            };

            let metadata = Self::metadata();
            AcpProvider::connect(metadata.name, goose_mode, provider_config).await
        })
    }
}
