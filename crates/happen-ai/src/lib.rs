mod provider;
mod orchestrator;
mod applicator;

pub use provider::{AiProvider, AiProviderConfig, AnthropicProvider};
pub use orchestrator::{AiOrchestrator, UserIntent};
pub use applicator::BlueprintApplicator;

use happen_core::{App, Plugin};

pub struct AiPlugin {
    pub config: AiProviderConfig,
}

impl Default for AiPlugin {
    fn default() -> Self {
        Self {
            config: AiProviderConfig::default(),
        }
    }
}

impl Plugin for AiPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AiState::new(self.config.clone()));
    }

    fn name(&self) -> &str {
        "AiPlugin"
    }
}

pub struct AiState {
    pub config: AiProviderConfig,
    pub last_error: Option<String>,
}

impl AiState {
    pub fn new(config: AiProviderConfig) -> Self {
        Self {
            config,
            last_error: None,
        }
    }
}

