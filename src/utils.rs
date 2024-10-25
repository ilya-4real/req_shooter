pub fn format_received_data_value(received_data_value: usize) -> String {
    if received_data_value > 1_000_000 {
        return format!("{:.2} MB", received_data_value as f32 / 1_000_000.0);
    }
    if received_data_value > 1_000 {
        format!("{:.2} KB", received_data_value as f32 / 1_000.0)
    } else {
        format!("{} B", received_data_value)
    }
}
