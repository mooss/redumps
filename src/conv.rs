pub fn to_mib<T: Into<f64>>(bytes: T) -> String {
    let mib = bytes.into() / (1024.0 * 1024.0);
    format!("{:.1}", mib)
}
