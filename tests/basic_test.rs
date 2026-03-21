use chetna::db::brain::{Brain, RecallWeights};

#[tokio::test]
async fn test_brain_initialization() {
    let brain = Brain::new(":memory:").unwrap();
    assert!(!brain.has_embedder().await);
}

#[tokio::test]
async fn test_memory_creation_and_retrieval() {
    let brain = Brain::new(":memory:").unwrap();
    let mem = brain.create_memory(
        "Test memory content",
        0.8,
        0.5,
        0.5,
        &["test".to_string()],
        "fact",
        "fact",
        None,
        None,
    ).await.unwrap();

    assert_eq!(mem.content, "Test memory content");
    assert!((mem.importance - 0.8).abs() < 0.001);
    assert_eq!(mem.tags, vec!["test".to_string()]);
}

#[tokio::test]
async fn test_recall_weights_default() {
    let weights = RecallWeights::default();
    assert_eq!(weights.similarity, 0.40);
    assert_eq!(weights.importance, 0.25);
    assert_eq!(weights.recency, 0.15);
    assert_eq!(weights.access_frequency, 0.10);
    assert_eq!(weights.emotional, 0.10);
}
