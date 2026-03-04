use crate::error::{AmpError, Result};
use std::sync::Arc;
use tokio::sync::mpsc;

/// Embedding request for the thread pool
pub struct EmbedRequest {
    pub text: String,
    pub response: tokio::sync::oneshot::Sender<Result<Vec<f32>>>,
}

/// Thread pool that runs ONNX inference off the async runtime
pub struct EmbeddingPool {
    sender: mpsc::Sender<EmbedRequest>,
}

impl EmbeddingPool {
    /// Create a new embedding pool with the given number of worker threads
    pub fn new(generator: Arc<dyn super::EmbeddingGenerator>, _num_threads: usize) -> Self {
        let (sender, mut receiver) = mpsc::channel::<EmbedRequest>(256);

        // Spawn a task that processes embedding requests
        let gen = generator;
        tokio::spawn(async move {
            while let Some(request) = receiver.recv().await {
                let gen = Arc::clone(&gen);
                let result = tokio::task::spawn_blocking(move || gen.embed(&request.text))
                    .await
                    .map_err(|e| AmpError::Embedding(format!("Thread pool error: {}", e)))
                    .and_then(|r| r);

                let _ = request.response.send(result);
            }
        });

        Self { sender }
    }

    /// Generate embedding asynchronously via the thread pool
    pub async fn embed(&self, text: String) -> Result<Vec<f32>> {
        let (tx, rx) = tokio::sync::oneshot::channel();
        self.sender
            .send(EmbedRequest { text, response: tx })
            .await
            .map_err(|_| AmpError::Embedding("Thread pool channel closed".into()))?;
        rx.await
            .map_err(|_| AmpError::Embedding("Thread pool response dropped".into()))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_embedding_pool_structure() {
        // Mock embedding generator that just returns dimension size
        struct MockEmbedding;

        impl super::super::EmbeddingGenerator for MockEmbedding {
            fn embed(&self, _text: &str) -> Result<Vec<f32>> {
                Ok(vec![0.0_f32; 384])
            }

            fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
                Ok(vec![vec![0.0_f32; 384]; texts.len()])
            }

            fn dimension(&self) -> usize {
                384
            }
        }

        let gen = Arc::new(MockEmbedding);
        let pool = EmbeddingPool::new(gen, 2);

        // Test that we can send a request to the pool
        let result = pool.embed("test".to_string()).await;
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384);
    }
}
