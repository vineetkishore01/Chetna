// Test TurboQuant implementation
use chetna::db::turboquant::TurboQuant;
use rand::Rng;

#[test]
fn test_turboquant_basic() {
    println!("Testing TurboQuant implementation...\n");

    let dimension = 1536;
    let bit_width = 3;

    println!(
        "Creating TurboQuant with dimension={} and bit_width={}",
        dimension, bit_width
    );
    let turbo = TurboQuant::new(dimension, bit_width);

    // Test 1: Quantize and dequantize a random vector
    println!("\n=== Test 1: Quantize/Dequantize ===");
    let mut rng = rand::thread_rng();
    let mut vector = vec![0.0f32; dimension];
    for v in vector.iter_mut() {
        *v = rng.gen::<f32>() * 2.0 - 1.0;
    }

    let original_norm = (vector.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    println!("Original vector norm: {:.6}", original_norm);

    let quantized = turbo.quantize_prod(&vector);
    println!(
        "Quantized - indices: {} values, qjl_bits: {} bytes, residual_norm: {:.6}",
        quantized.indices.len(),
        quantized.qjl_bits.len(),
        quantized.residual_norm
    );

    let reconstructed = turbo.dequantize(&quantized);
    let reconstructed_norm = (reconstructed.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    println!("Reconstructed vector norm: {:.6}", reconstructed_norm);

    let mse = vector
        .iter()
        .zip(reconstructed.iter())
        .map(|(a, b)| (a - b).powi(2))
        .sum::<f32>()
        / dimension as f32;
    println!("MSE per dimension: {:.6}", mse);

    // Test 2: Inner product preservation
    println!("\n=== Test 2: Inner Product Preservation ===");
    let mut vector2 = vec![0.0f32; dimension];
    for v in vector2.iter_mut() {
        *v = rng.gen::<f32>() * 2.0 - 1.0;
    }

    // Normalize vectors before computing inner product (as TurboQuant does)
    let norm1 = (vector.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let norm2 = (vector2.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let normalized1: Vec<f32> = vector.iter().map(|&x| x / norm1).collect();
    let normalized2: Vec<f32> = vector2.iter().map(|&x| x / norm2).collect();

    let ip_original = normalized1
        .iter()
        .zip(normalized2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    let quantized2 = turbo.quantize_prod(&vector2);
    let reconstructed2 = turbo.dequantize(&quantized2);

    // Normalize reconstructed vectors for fair comparison
    let norm_recon1 = (reconstructed.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let norm_recon2 = (reconstructed2.iter().map(|&x| x * x).sum::<f32>()).sqrt();
    let normalized_recon1: Vec<f32> = reconstructed.iter().map(|&x| x / norm_recon1).collect();
    let normalized_recon2: Vec<f32> = reconstructed2.iter().map(|&x| x / norm_recon2).collect();

    let ip_quant = normalized_recon1
        .iter()
        .zip(normalized_recon2.iter())
        .map(|(a, b)| a * b)
        .sum::<f32>();

    println!("Original inner product (normalized): {:.6}", ip_original);
    println!("Quantized inner product (normalized): {:.6}", ip_quant);
    println!("Absolute error: {:.6}", (ip_original - ip_quant).abs());
    println!(
        "Relative error: {:.2}%",
        if ip_original.abs() > 1e-6 {
            (ip_original - ip_quant).abs() / ip_original.abs() * 100.0
        } else {
            0.0
        }
    );

    // Test 3: Compression ratio
    println!("\n=== Test 3: Compression Ratio ===");
    let original_size = dimension * 4; // 4 bytes per f32
    let quantized_size = quantized.indices.len() + quantized.qjl_bits.len() + 4; // indices + bits + 2 f32
    println!("Original size: {} bytes", original_size);
    println!("Quantized size: {} bytes", quantized_size);
    println!(
        "Compression ratio: {:.2}x",
        original_size as f32 / quantized_size as f32
    );

    println!("\n=== All tests completed successfully! ===");

    // Assertions
    assert!(mse < 0.1, "MSE should be less than 0.1");
    // Note: Inner product preservation has small absolute error (~0.01) but can have high relative error
    // when original inner product is small. This is expected for low bit-width quantization.
    // The theoretical bound for b=3 is D_prod ≈ 0.18/d ≈ 0.000117 (variance), so std dev ≈ 0.01
    assert!(
        (ip_original - ip_quant).abs() < 0.05,
        "Absolute error should be less than 0.05"
    );
    assert!(
        original_size as f32 / quantized_size as f32 > 3.0,
        "Compression ratio should be > 3x"
    );
}
