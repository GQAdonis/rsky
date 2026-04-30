use serde::{Deserialize, Serialize};

use super::feed::GeneratorView;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GetPopularFeedGeneratorsOutput {
    pub feeds: Vec<GeneratorView>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
}
