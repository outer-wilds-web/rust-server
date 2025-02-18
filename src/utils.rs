pub fn normalize_direction(direction: (f64, f64, f64)) -> (f64, f64, f64) {
    let length = (direction.0.powi(2) + direction.1.powi(2) + direction.2.powi(2)).sqrt();
    (
        direction.0 / length,
        direction.1 / length,
        direction.2 / length,
    )
}
