//! Shared utilities for Chetna

use anyhow::Result;

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a == 0.0 || mag_b == 0.0 {
        return 0.0;
    }
    dot / (mag_a * mag_b)
}

pub fn vec_to_blob(vec: &[f32]) -> Vec<u8> {
    let mut blob = Vec::with_capacity(vec.len() * 4);
    for &val in vec {
        blob.extend_from_slice(&val.to_le_bytes());
    }
    blob
}

pub fn blob_to_vec(blob: &[u8]) -> Result<Vec<f32>> {
    let float_count = blob.len() / 4;
    let mut vec = vec![0.0; float_count];
    for i in 0..float_count {
        let bytes: [u8; 4] = blob[i * 4..(i + 1) * 4].try_into()?;
        vec[i] = f32::from_le_bytes(bytes);
    }
    Ok(vec)
}
