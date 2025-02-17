// Sigmoid function, result should be in the range [0; 1]
pub fn sigmoid(x: f32, scaling_factor: f32) -> f32 {
    1.0 / (1.0 + (-x / scaling_factor).exp())
}

// Sigmoid function, with specified range
pub fn sigmoid_ranged(x: f32, scaling_factor: f32, min: f32, max: f32) -> f32 {
    max * sigmoid(x, scaling_factor) - min
}

// computes multi-level sigmoid function, each level has it's own range and scaling factor
pub fn multi_level_sigmoid(level_values: Vec<(f32, f32, f32, f32)>) -> f32 {
    let mut result: f32 = 0.0;
    for values in level_values.iter() {
        result += sigmoid_ranged(values.0, values.1, values.2, values.3);
    }
    result
}
