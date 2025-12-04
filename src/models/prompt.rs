//! Prompt Models
//!
//! Data structures for prompt rendering and template management.

use serde::{Deserialize, Serialize};

/// Context for rendering prompt templates
///
/// Contains dynamic values that will be substituted into prompt templates
/// using placeholder patterns like {question}, {mood}, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PromptRenderContext {
    /// User's question (required)
    pub question: String,
    /// Emotional tone/mood of the question (optional)
    pub mood: Option<String>,
    /// Main topic or category of the question (optional)
    pub topic: Option<String>,
    /// Time period relevant to the question (optional)
    pub period: Option<String>,
    /// List of tarot cards drawn (optional, for reading agent)
    pub cards: Option<Vec<String>>,
}

impl PromptRenderContext {
    /// Create a new context with just a question (required field)
    pub fn new(question: String) -> Self {
        Self {
            question,
            mood: None,
            topic: None,
            period: None,
            cards: None,
        }
    }

    /// Add mood to the context
    pub fn with_mood(mut self, mood: String) -> Self {
        self.mood = Some(mood);
        self
    }

    /// Add topic to the context
    pub fn with_topic(mut self, topic: String) -> Self {
        self.topic = Some(topic);
        self
    }

    /// Add period to the context
    pub fn with_period(mut self, period: String) -> Self {
        self.period = Some(period);
        self
    }

    /// Add cards to the context (for reading agent)
    pub fn with_cards(mut self, cards: Vec<String>) -> Self {
        self.cards = Some(cards);
        self
    }

    /// Get all placeholder values as a map for template rendering
    pub fn as_placeholder_map(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::new();
        map.insert("question".to_string(), self.question.clone());

        if let Some(ref mood) = self.mood {
            map.insert("mood".to_string(), mood.clone());
        }

        if let Some(ref topic) = self.topic {
            map.insert("topic".to_string(), topic.clone());
        }

        if let Some(ref period) = self.period {
            map.insert("period".to_string(), period.clone());
        }

        if let Some(ref cards) = self.cards {
            map.insert("cards".to_string(), cards.join(", "));
        }

        map
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_render_context_builder() {
        let context = PromptRenderContext::new("What is my future?".to_string())
            .with_mood("anxious".to_string())
            .with_topic("career".to_string())
            .with_period("next_year".to_string())
            .with_cards(vec!["The Sun".to_string(), "The Moon".to_string()]);

        assert_eq!(context.question, "What is my future?");
        assert_eq!(context.mood, Some("anxious".to_string()));
        assert_eq!(context.topic, Some("career".to_string()));
        assert_eq!(context.period, Some("next_year".to_string()));
        assert_eq!(
            context.cards,
            Some(vec!["The Sun".to_string(), "The Moon".to_string()])
        );
    }

    #[test]
    fn test_placeholder_map_generation() {
        let context =
            PromptRenderContext::new("test question".to_string()).with_mood("happy".to_string());

        let map = context.as_placeholder_map();

        assert_eq!(map.get("question"), Some(&"test question".to_string()));
        assert_eq!(map.get("mood"), Some(&"happy".to_string()));
        assert_eq!(map.get("topic"), None);
        assert_eq!(map.get("period"), None);
        assert_eq!(map.get("cards"), None);
    }

    #[test]
    fn test_cards_placeholder_formatting() {
        let context = PromptRenderContext::new("test".to_string()).with_cards(vec![
            "Card 1".to_string(),
            "Card 2".to_string(),
            "Card 3".to_string(),
        ]);

        let map = context.as_placeholder_map();
        assert_eq!(
            map.get("cards"),
            Some(&"Card 1, Card 2, Card 3".to_string())
        );
    }
}
