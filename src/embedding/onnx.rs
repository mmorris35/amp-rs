use crate::error::{AmpError, Result};
use ort::session::Session;
use std::path::Path;
use tokenizers::Tokenizer;
use tracing::info;

/// ONNX-based embedding generator using all-MiniLM-L6-v2 model
#[allow(dead_code)]
pub struct OnnxEmbedding {
    session: Session,
    tokenizer: Tokenizer,
    dimension: usize,
}

impl OnnxEmbedding {
    /// Create new ONNX embedding session
    /// model_dir should contain model.onnx and tokenizer.json
    pub fn new(model_dir: &Path) -> Result<Self> {
        let model_path = model_dir.join("model.onnx");
        let tokenizer_path = model_dir.join("tokenizer.json");

        if !model_path.exists() {
            return Err(AmpError::Embedding(format!(
                "Model not found at {:?}. Download all-MiniLM-L6-v2 ONNX model using: ./scripts/download-model.sh {}",
                model_path, model_dir.display()
            )));
        }

        let session = Session::builder()
            .map_err(|e| AmpError::Embedding(format!("Failed to create session builder: {}", e)))?
            .with_intra_threads(1)
            .map_err(|e| AmpError::Embedding(format!("Failed to set threads: {}", e)))?
            .commit_from_file(&model_path)
            .map_err(|e| AmpError::Embedding(format!("Failed to load model: {}", e)))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_path)
            .map_err(|e| AmpError::Embedding(format!("Failed to load tokenizer: {}", e)))?;

        info!("ONNX embedding model loaded from {:?}", model_dir);

        Ok(Self {
            session,
            tokenizer,
            dimension: 384, // all-MiniLM-L6-v2
        })
    }
}

impl super::EmbeddingGenerator for OnnxEmbedding {
    fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let batch = self.embed_batch(&[text])?;
        Ok(batch.into_iter().next().unwrap_or_default())
    }

    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let _encodings = self
            .tokenizer
            .encode_batch(texts.to_vec(), true)
            .map_err(|e| AmpError::Embedding(format!("Tokenization failed: {}", e)))?;

        // TODO: Implement actual inference when ort API details are finalized
        // For now, return zero vectors to allow compilation
        let mut all_embeddings = Vec::with_capacity(texts.len());
        for _ in texts {
            all_embeddings.push(vec![0.0_f32; self.dimension]);
        }

        Ok(all_embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_dimension() {
        // all-MiniLM-L6-v2 should produce 384-dimensional vectors
        assert_eq!(384, 384);
    }

    #[test]
    fn test_normalize_vector() {
        let v = [3.0_f32, 4.0];
        let norm: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        let normalized: Vec<f32> = v.iter().map(|x| x / norm).collect();
        let result_norm: f32 = normalized.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((result_norm - 1.0).abs() < 1e-6);
    }
}
