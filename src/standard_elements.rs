use lotus_script::time::delta;

pub fn exponential_approach(old_value: f32, exponent: f32, target: f32) -> f32 {
    (1.0 - (delta() * -exponent).exp()) * (target - old_value) + old_value
}
