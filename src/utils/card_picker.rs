pub fn pick_card_count() -> usize {
    // Simple random chooser: 3 or 5
    // In unit tests we should call a deterministic helper instead
    if rand::random() {
        3
    } else {
        5
    }
}
