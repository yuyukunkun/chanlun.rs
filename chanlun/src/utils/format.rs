/// Format f64 with Python :g semantics — strip trailing zeros, no scientific notation for common values
pub fn format_f64_g(value: f64) -> String {
    if value.is_nan() {
        return "nan".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "inf".to_string() } else { "-inf".to_string() };
    }

    // Use high precision then trim trailing zeros
    let s = format!("{:.15}", value);
    let s = s.trim_end_matches('0');
    s.trim_end_matches('.').to_string()
}
