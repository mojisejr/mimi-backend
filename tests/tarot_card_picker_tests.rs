#[cfg(test)]
mod tests {
    use mimivibe_backend::utils::card_picker;

    #[test]
    fn returns_3_or_5() {
        let c = card_picker::pick_card_count();
        assert!(c == 3 || c == 5, "card_picker must return 3 or 5");
    }
}
