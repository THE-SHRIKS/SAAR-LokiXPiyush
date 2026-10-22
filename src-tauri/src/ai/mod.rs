pub mod classifier;
pub mod defense;
pub mod grounding;
pub mod llm_engine;
pub mod search;

pub use classifier::{CategoryProposal, TransactionClassifier, ALLOWED_CATEGORIES};
pub use defense::AdversarialDefense;
pub use grounding::{GroundedChatResponse, GroundedFactPacket, GroundingEngine, QueryIntent};
pub use llm_engine::{LlmConfig, LlmEngine, LlmError, EXPECTED_MODEL_SHA256};
pub use search::{SearchEngine, SearchResult};
