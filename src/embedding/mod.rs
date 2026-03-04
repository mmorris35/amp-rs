pub mod onnx;
pub mod pool;

use crate::error::Result;

/// Trait for generating embeddings from text
pub trait EmbeddingGenerator: Send + Sync {
    /// Generate embedding vector for a single text
    fn embed(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts (batch)
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Get the embedding dimension
    fn dimension(&self) -> usize;
}
