pub fn to_mib<T: Into<f64>>(bytes: T) -> f64 {
    bytes.into() / (1024.0 * 1024.0)
}
