//! Topic mapping and wildcard matching

use crate::models::{MappingDirection, TopicMapping};

/// Topic mapper for MQTT/ZeroMQ topic conversion
pub struct TopicMapper {
    mappings: Vec<TopicMapping>,
}

impl TopicMapper {
    pub fn new(mappings: Vec<TopicMapping>) -> Self {
        Self { mappings }
    }

    /// Update mappings
    pub fn update_mappings(&mut self, mappings: Vec<TopicMapping>) {
        self.mappings = mappings;
    }

    /// Get all enabled MQTT source topics (for subscription)
    pub fn get_mqtt_subscribe_topics(&self) -> Vec<String> {
        self.mappings
            .iter()
            .filter(|m| {
                m.enabled
                    && (m.direction == MappingDirection::MqttToZmq
                        || m.direction == MappingDirection::Bidirectional)
            })
            .map(|m| m.source_topic.clone())
            .collect()
    }

    /// Match a source topic and return the target topic for MQTT → ZMQ
    pub fn map_mqtt_to_zmq(&self, source_topic: &str) -> Option<String> {
        for mapping in &self.mappings {
            if !mapping.enabled {
                continue;
            }
            if mapping.direction != MappingDirection::MqttToZmq
                && mapping.direction != MappingDirection::Bidirectional
            {
                continue;
            }

            if matches_topic_pattern(&mapping.source_topic, source_topic) {
                return Some(apply_topic_mapping(
                    &mapping.source_topic,
                    &mapping.target_topic,
                    source_topic,
                ));
            }
        }
        None
    }

    /// Match a source topic and return the target topic for ZMQ → MQTT
    pub fn map_zmq_to_mqtt(&self, source_topic: &str) -> Option<String> {
        for mapping in &self.mappings {
            if !mapping.enabled {
                continue;
            }
            if mapping.direction != MappingDirection::ZmqToMqtt
                && mapping.direction != MappingDirection::Bidirectional
            {
                continue;
            }

            // For ZMQ→MQTT, we match against source_topic pattern
            if matches_topic_pattern(&mapping.source_topic, source_topic) {
                return Some(apply_topic_mapping(
                    &mapping.source_topic,
                    &mapping.target_topic,
                    source_topic,
                ));
            }
        }
        None
    }
}

/// Check if a topic matches a pattern with MQTT wildcards
/// + matches single level
/// # matches multiple levels (only at end)
fn matches_topic_pattern(pattern: &str, topic: &str) -> bool {
    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let topic_parts: Vec<&str> = topic.split('/').collect();

    let mut p_idx = 0;
    let mut t_idx = 0;

    while p_idx < pattern_parts.len() && t_idx < topic_parts.len() {
        let p = pattern_parts[p_idx];
        let t = topic_parts[t_idx];

        if p == "#" {
            // # matches everything from here
            return true;
        } else if p == "+" {
            // + matches single level
            p_idx += 1;
            t_idx += 1;
        } else if p == t {
            p_idx += 1;
            t_idx += 1;
        } else {
            return false;
        }
    }

    // Check if we've consumed all parts
    if p_idx == pattern_parts.len() && t_idx == topic_parts.len() {
        return true;
    }

    // Check if remaining pattern is just #
    if p_idx < pattern_parts.len() && pattern_parts[p_idx] == "#" {
        return true;
    }

    false
}

/// Apply topic mapping, preserving wildcard-matched segments
fn apply_topic_mapping(pattern: &str, target: &str, source: &str) -> String {
    // If target doesn't contain wildcards and pattern does,
    // we need to preserve the matched portions

    if !pattern.contains('+') && !pattern.contains('#') {
        // Exact match pattern, just return target
        return target.to_string();
    }

    // For now, simple replacement - can be enhanced for complex mappings
    // If pattern has wildcards, we extract matched parts and substitute

    let _pattern_parts: Vec<&str> = pattern.split('/').collect();
    let source_parts: Vec<&str> = source.split('/').collect();
    let target_parts: Vec<&str> = target.split('/').collect();

    let mut result_parts: Vec<String> = Vec::new();
    let mut source_idx = 0;

    for tp in &target_parts {
        if *tp == "+" && source_idx < source_parts.len() {
            result_parts.push(source_parts[source_idx].to_string());
            source_idx += 1;
        } else if *tp == "#" {
            // Append all remaining source parts
            while source_idx < source_parts.len() {
                result_parts.push(source_parts[source_idx].to_string());
                source_idx += 1;
            }
        } else {
            result_parts.push((*tp).to_string());
        }
    }

    // If target has fewer parts and no wildcards, just use source topic parts for remaining
    if result_parts.is_empty() {
        return target.to_string();
    }

    result_parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exact_match() {
        assert!(matches_topic_pattern("sensors/temperature", "sensors/temperature"));
        assert!(!matches_topic_pattern("sensors/temperature", "sensors/humidity"));
    }

    #[test]
    fn test_single_level_wildcard() {
        assert!(matches_topic_pattern("sensors/+/temperature", "sensors/room1/temperature"));
        assert!(matches_topic_pattern("sensors/+", "sensors/room1"));
        assert!(!matches_topic_pattern("sensors/+", "sensors/room1/temperature"));
    }

    #[test]
    fn test_multi_level_wildcard() {
        assert!(matches_topic_pattern("sensors/#", "sensors/room1/temperature"));
        assert!(matches_topic_pattern("sensors/#", "sensors"));
        assert!(matches_topic_pattern("#", "anything/goes/here"));
    }
}
