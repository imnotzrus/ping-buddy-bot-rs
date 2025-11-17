//! Input validation utilities.
//!
//! This module provides validation functions for user inputs,
//! ensuring data integrity and security.

use crate::constants::MAX_TOPIC_LENGTH;
use crate::constants::RESERVED_TOPICS;
use crate::error::TopicValidationError;

/// Validates a topic name according to the following rules:
/// - Not empty
/// - Length <= MAX_TOPIC_LENGTH
/// - Contains only alphanumeric characters and underscores
/// - Starts with a letter
/// - Not a reserved name
pub fn validate_topic(topic: &str) -> Result<(), TopicValidationError> {
  if topic.is_empty() {
    return Err(TopicValidationError::Empty);
  }

  if topic.len() > MAX_TOPIC_LENGTH {
    return Err(TopicValidationError::TooLong {
      max: MAX_TOPIC_LENGTH,
      actual: topic.len(),
    });
  }

  // Check if topic name is reserved
  if RESERVED_TOPICS.contains(&topic.to_lowercase().as_str()) {
    return Err(TopicValidationError::Reserved(topic.to_string()));
  }

  // Check if all characters are alphanumeric or underscore
  if !topic.chars().all(|c| c.is_alphanumeric() || c == '_') {
    return Err(TopicValidationError::InvalidCharacters);
  }

  // Check if first character is alphabetic
  if let Some(first_char) = topic.chars().next() {
    if !first_char.is_alphabetic() {
      return Err(TopicValidationError::InvalidStart);
    }
  }

  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_valid_topics() {
    assert!(validate_topic("general").is_ok());
    assert!(validate_topic("dev_team").is_ok());
    assert!(validate_topic("project123").is_ok());
    assert!(validate_topic("a").is_ok());
  }

  #[test]
  fn test_invalid_topics() {
    // Empty
    assert!(matches!(
      validate_topic(""),
      Err(TopicValidationError::Empty)
    ));

    // Invalid start
    assert!(matches!(
      validate_topic("123topic"),
      Err(TopicValidationError::InvalidStart)
    ));

    // Invalid characters
    assert!(matches!(
      validate_topic("topic-name"),
      Err(TopicValidationError::InvalidCharacters)
    ));
    assert!(matches!(
      validate_topic("topic name"),
      Err(TopicValidationError::InvalidCharacters)
    ));

    // Reserved
    assert!(matches!(
      validate_topic("all"),
      Err(TopicValidationError::Reserved(_))
    ));

    // Too long
    assert!(matches!(
      validate_topic(&"a".repeat(51)),
      Err(TopicValidationError::TooLong { .. })
    ));
  }
}
