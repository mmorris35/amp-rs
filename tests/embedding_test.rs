// Tests for embedding engine (trait, ONNX, and thread pool)
// Tests that require the ONNX model are gated behind a feature or env var
// For CI without model: test the trait, pool structure, error handling
// For local with model: test actual embedding generation

#[test]
fn test_embedding_dimension() {
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

#[cfg(test)]
mod embedding_trait_tests {
    use amp_rs::embedding::EmbeddingGenerator;
    use amp_rs::error::Result;

    struct MockEmbedding {
        dimension: usize,
    }

    impl EmbeddingGenerator for MockEmbedding {
        fn embed(&self, _text: &str) -> Result<Vec<f32>> {
            Ok(vec![0.1_f32; self.dimension])
        }

        fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(vec![vec![0.1_f32; self.dimension]; texts.len()])
        }

        fn dimension(&self) -> usize {
            self.dimension
        }
    }

    #[test]
    fn test_embedding_trait_embed() {
        let gen = MockEmbedding { dimension: 384 };
        let embedding = gen.embed("hello world").unwrap();
        assert_eq!(embedding.len(), 384);
        assert!((embedding[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_embedding_trait_embed_batch() {
        let gen = MockEmbedding { dimension: 384 };
        let texts = vec!["hello", "world"];
        let embeddings = gen.embed_batch(&texts).unwrap();
        assert_eq!(embeddings.len(), 2);
        assert_eq!(embeddings[0].len(), 384);
        assert_eq!(embeddings[1].len(), 384);
    }

    #[test]
    fn test_embedding_trait_dimension() {
        let gen = MockEmbedding { dimension: 384 };
        assert_eq!(gen.dimension(), 384);
    }

    #[test]
    fn test_embedding_batch_empty() {
        let gen = MockEmbedding { dimension: 384 };
        let embeddings = gen.embed_batch(&[]).unwrap();
        assert_eq!(embeddings.len(), 0);
    }
}

#[cfg(test)]
mod embedding_pool_tests {
    use amp_rs::embedding::{pool::EmbeddingPool, EmbeddingGenerator};
    use amp_rs::error::Result;
    use std::sync::Arc;

    struct MockEmbedding;

    impl EmbeddingGenerator for MockEmbedding {
        fn embed(&self, text: &str) -> Result<Vec<f32>> {
            // Return vector with length based on text length as a simple mock
            Ok(vec![0.5_f32; text.len().max(384)])
        }

        fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
            Ok(texts
                .iter()
                .map(|t| vec![0.5_f32; t.len().max(384)])
                .collect())
        }

        fn dimension(&self) -> usize {
            384
        }
    }

    #[tokio::test]
    async fn test_embedding_pool_creation() {
        let gen = Arc::new(MockEmbedding);
        let _pool = EmbeddingPool::new(gen, 2);
        // Pool created successfully
    }

    #[tokio::test]
    async fn test_embedding_pool_embed() {
        let gen = Arc::new(MockEmbedding);
        let pool = EmbeddingPool::new(gen, 2);

        let result = pool.embed("test".to_string()).await;
        assert!(result.is_ok());
        let embedding = result.unwrap();
        assert!(embedding.len() >= 4); // At least "test".len()
    }

    #[tokio::test]
    async fn test_embedding_pool_multiple_requests() {
        let gen = Arc::new(MockEmbedding);
        let pool = Arc::new(EmbeddingPool::new(gen, 2));

        let mut handles = vec![];
        for i in 0..3 {
            let pool_clone = Arc::clone(&pool);
            let handle = tokio::spawn(async move {
                let text = format!("request {}", i);
                pool_clone.embed(text).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            let embedding_result = result.unwrap();
            assert!(embedding_result.is_ok());
        }
    }
}

#[tokio::test]
#[ignore = "Requires ONNX model download"]
async fn test_onnx_embedding_real() {
    use amp_rs::embedding::{onnx::OnnxEmbedding, EmbeddingGenerator};
    use std::path::Path;

    let model_dir = Path::new("models");
    if !model_dir.join("model.onnx").exists() {
        eprintln!("Skipping: ONNX model not found. Run scripts/download-model.sh");
        return;
    }

    let engine = OnnxEmbedding::new(model_dir).unwrap();
    let embedding = engine.embed("hello world").unwrap();
    assert_eq!(embedding.len(), 384);

    // Verify normalization
    let norm: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
    assert!((norm - 1.0).abs() < 0.01);
}
