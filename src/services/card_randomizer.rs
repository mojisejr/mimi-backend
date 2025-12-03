//! Card Randomizer Service
//!
//! Provides tarot card selection functionality with proper randomization
//! and validation for 3-card and 5-card readings.

use rand::{seq::SliceRandom, SeedableRng};
use thiserror::Error;

/// Error types for card randomization
#[derive(Debug, Error)]
pub enum CardRandomizerError {
    #[error("Invalid card count: {0}. Must be 3 or 5.")]
    InvalidCardCount(u32),
    #[error("Cannot select {0} unique cards from deck of {1}")]
    InsufficientCards(u32, usize),
    #[error("Tarot deck is empty")]
    EmptyDeck,
}

/// Complete Tarot deck (78 cards)
const TAROT_DECK: &[&str] = &[
    // Major Arcana (22 cards)
    "The Fool",
    "The Magician",
    "The High Priestess",
    "The Empress",
    "The Emperor",
    "The Hierophant",
    "The Lovers",
    "The Chariot",
    "Strength",
    "The Hermit",
    "Wheel of Fortune",
    "Justice",
    "The Hanged Man",
    "Death",
    "Temperance",
    "The Devil",
    "The Tower",
    "The Star",
    "The Moon",
    "The Sun",
    "Judgement",
    "The World",
    // Minor Arcana - Wands (14 cards)
    "Ace of Wands",
    "Two of Wands",
    "Three of Wands",
    "Four of Wands",
    "Five of Wands",
    "Six of Wands",
    "Seven of Wands",
    "Eight of Wands",
    "Nine of Wands",
    "Ten of Wands",
    "Page of Wands",
    "Knight of Wands",
    "Queen of Wands",
    "King of Wands",
    // Minor Arcana - Cups (14 cards)
    "Ace of Cups",
    "Two of Cups",
    "Three of Cups",
    "Four of Cups",
    "Five of Cups",
    "Six of Cups",
    "Seven of Cups",
    "Eight of Cups",
    "Nine of Cups",
    "Ten of Cups",
    "Page of Cups",
    "Knight of Cups",
    "Queen of Cups",
    "King of Cups",
    // Minor Arcana - Swords (14 cards)
    "Ace of Swords",
    "Two of Swords",
    "Three of Swords",
    "Four of Swords",
    "Five of Swords",
    "Six of Swords",
    "Seven of Swords",
    "Eight of Swords",
    "Nine of Swords",
    "Ten of Swords",
    "Page of Swords",
    "Knight of Swords",
    "Queen of Swords",
    "King of Swords",
    // Minor Arcana - Pentacles (14 cards)
    "Ace of Pentacles",
    "Two of Pentacles",
    "Three of Pentacles",
    "Four of Pentacles",
    "Five of Pentacles",
    "Six of Pentacles",
    "Seven of Pentacles",
    "Eight of Pentacles",
    "Nine of Pentacles",
    "Ten of Pentacles",
    "Page of Pentacles",
    "Knight of Pentacles",
    "Queen of Pentacles",
    "King of Pentacles",
];

/// Card Randomizer for tarot readings
#[derive(Debug, Clone)]
pub struct CardRandomizer {
    rng: rand::rngs::ThreadRng,
}

impl CardRandomizer {
    /// Create a new CardRandomizer instance
    pub fn new() -> Self {
        Self {
            rng: rand::thread_rng(),
        }
    }

    /// Pick a specified number of unique cards from the tarot deck
    ///
    /// # Arguments
    ///
    /// * `count` - Number of cards to pick (must be 3 or 5)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Vector of unique card names
    /// * `Err(CardRandomizerError)` - Error if invalid count or insufficient cards
    pub async fn pick_cards(&mut self, count: u32) -> Result<Vec<String>, CardRandomizerError> {
        // Validate card count
        if count != 3 && count != 5 {
            return Err(CardRandomizerError::InvalidCardCount(count));
        }

        // Ensure we have enough cards in the deck
        if TAROT_DECK.len() < count as usize {
            return Err(CardRandomizerError::InsufficientCards(
                count,
                TAROT_DECK.len(),
            ));
        }

        // Check if deck is empty (unlikely since we use a const array)
        #[allow(clippy::const_is_empty)]
        if TAROT_DECK.is_empty() {
            return Err(CardRandomizerError::EmptyDeck);
        }

        // Create a mutable copy of the deck
        let mut deck: Vec<&str> = TAROT_DECK.to_vec();

        // Shuffle the deck
        deck.shuffle(&mut self.rng);

        // Select the required number of cards
        let selected_cards: Vec<String> = deck
            .into_iter()
            .take(count as usize)
            .map(|card| card.to_string())
            .collect();

        // Verify we got the expected number of unique cards
        if selected_cards.len() != count as usize {
            return Err(CardRandomizerError::InsufficientCards(
                count,
                selected_cards.len(),
            ));
        }

        // Verify all cards are unique
        let unique_cards: std::collections::HashSet<_> = selected_cards.iter().collect();
        if unique_cards.len() != selected_cards.len() {
            return Err(CardRandomizerError::InsufficientCards(
                count,
                unique_cards.len(),
            ));
        }

        Ok(selected_cards)
    }

    /// Pick cards with deterministic seed for testing
    ///
    /// # Arguments
    ///
    /// * `count` - Number of cards to pick (must be 3 or 5)
    /// * `seed` - Seed for deterministic randomization (for testing)
    ///
    /// # Returns
    ///
    /// * `Ok(Vec<String>)` - Vector of unique card names
    /// * `Err(CardRandomizerError)` - Error if invalid count or insufficient cards
    pub async fn pick_cards_seeded(
        &mut self,
        count: u32,
        seed: u64,
    ) -> Result<Vec<String>, CardRandomizerError> {
        // Create deterministic RNG for testing
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(seed);

        // Validate card count
        if count != 3 && count != 5 {
            return Err(CardRandomizerError::InvalidCardCount(count));
        }

        // Create a mutable copy of the deck
        let mut deck: Vec<&str> = TAROT_DECK.to_vec();

        // Shuffle with seeded RNG
        deck.shuffle(&mut seeded_rng);

        // Select the required number of cards
        let selected_cards: Vec<String> = deck
            .into_iter()
            .take(count as usize)
            .map(|card| card.to_string())
            .collect();

        Ok(selected_cards)
    }

    /// Get the total number of cards in the tarot deck
    pub fn deck_size() -> usize {
        TAROT_DECK.len()
    }

    /// Check if a card name is valid (exists in the tarot deck)
    pub fn is_valid_card(card_name: &str) -> bool {
        TAROT_DECK.contains(&card_name)
    }

    /// Get all major arcana cards
    pub fn get_major_arcana() -> Vec<&'static str> {
        TAROT_DECK.iter().take(22).copied().collect()
    }

    /// Get all minor arcana cards
    pub fn get_minor_arcana() -> Vec<&'static str> {
        TAROT_DECK.iter().skip(22).copied().collect()
    }
}

impl Default for CardRandomizer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_pick_cards_3() {
        let mut randomizer = CardRandomizer::new();
        let cards = randomizer.pick_cards(3).await.unwrap();

        assert_eq!(cards.len(), 3);

        // Verify all cards are unique
        let unique_cards: std::collections::HashSet<_> = cards.iter().collect();
        assert_eq!(unique_cards.len(), 3);

        // Verify all cards are from the tarot deck
        for card in &cards {
            assert!(CardRandomizer::is_valid_card(card));
        }
    }

    #[tokio::test]
    async fn test_pick_cards_5() {
        let mut randomizer = CardRandomizer::new();
        let cards = randomizer.pick_cards(5).await.unwrap();

        assert_eq!(cards.len(), 5);

        // Verify all cards are unique
        let unique_cards: std::collections::HashSet<_> = cards.iter().collect();
        assert_eq!(unique_cards.len(), 5);

        // Verify all cards are from the tarot deck
        for card in &cards {
            assert!(CardRandomizer::is_valid_card(card));
        }
    }

    #[tokio::test]
    async fn test_pick_cards_invalid_count() {
        let mut randomizer = CardRandomizer::new();

        // Test invalid counts
        let result = randomizer.pick_cards(0).await;
        assert!(result.is_err());

        let result = randomizer.pick_cards(1).await;
        assert!(result.is_err());

        let result = randomizer.pick_cards(4).await;
        assert!(result.is_err());

        let result = randomizer.pick_cards(10).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_pick_cards_seeded() {
        let mut randomizer = CardRandomizer::new();
        let seed = 42;

        let cards1 = randomizer.pick_cards_seeded(3, seed).await.unwrap();
        let cards2 = randomizer.pick_cards_seeded(3, seed).await.unwrap();

        // Should be the same due to same seed
        assert_eq!(cards1, cards2);
    }

    #[test]
    fn test_deck_size() {
        assert_eq!(CardRandomizer::deck_size(), 78);
    }

    #[test]
    fn test_is_valid_card() {
        assert!(CardRandomizer::is_valid_card("The Fool"));
        assert!(CardRandomizer::is_valid_card("Ace of Wands"));
        assert!(!CardRandomizer::is_valid_card("Invalid Card"));
        assert!(!CardRandomizer::is_valid_card(""));
    }

    #[test]
    fn test_major_arcana() {
        let major = CardRandomizer::get_major_arcana();
        assert_eq!(major.len(), 22);
        assert!(major.contains(&"The Fool"));
        assert!(major.contains(&"The World"));
        assert!(!major.contains(&"Ace of Wands"));
    }

    #[test]
    fn test_minor_arcana() {
        let minor = CardRandomizer::get_minor_arcana();
        assert_eq!(minor.len(), 56); // 78 total - 22 major
        assert!(!minor.contains(&"The Fool"));
        assert!(minor.contains(&"Ace of Wands"));
        assert!(minor.contains(&"King of Pentacles"));
    }
}
