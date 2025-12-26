//! Tests for brivas-im-sdk

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    mod message_tests {
        use crate::message::{Message, MessageContent, MessageType};
        use uuid::Uuid;

        #[test]
        fn test_create_text_message() {
            let conv_id = Uuid::new_v4();
            let sender_id = Uuid::new_v4();
            let message = Message::new_text(conv_id, sender_id, "Hello, World!".to_string());
            
            assert_eq!(message.conversation_id, conv_id);
            assert_eq!(message.sender_id, sender_id);
            assert_eq!(message.message_type, MessageType::Text);
            assert!(!message.encrypted);
            assert!(message.reactions.is_empty());
        }

        #[test]
        fn test_message_content_variants() {
            let text = MessageContent::Text {
                text: "Hello".to_string(),
                mentions: vec![],
            };
            
            let image = MessageContent::Image {
                url: "https://cdn.brivas.io/image.jpg".to_string(),
                thumbnail_url: "https://cdn.brivas.io/thumb.jpg".to_string(),
                width: 1920,
                height: 1080,
                caption: Some("Test image".to_string()),
            };

            let location = MessageContent::Location {
                latitude: 6.5244,
                longitude: 3.3792,
                name: Some("Lagos, Nigeria".to_string()),
                address: None,
            };

            // Verify serialization
            let text_json = serde_json::to_string(&text).unwrap();
            assert!(text_json.contains("Hello"));

            let image_json = serde_json::to_string(&image).unwrap();
            assert!(image_json.contains("image"));

            let location_json = serde_json::to_string(&location).unwrap();
            assert!(location_json.contains("6.5244"));
        }
    }

    mod conversation_tests {
        use crate::conversation::{Conversation, ConversationType, ParticipantRole};
        use uuid::Uuid;

        #[test]
        fn test_create_direct_conversation() {
            let user1 = Uuid::new_v4();
            let user2 = Uuid::new_v4();
            let conv = Conversation::new_direct(user1, user2);
            
            assert!(matches!(conv.conversation_type, ConversationType::Direct));
            assert_eq!(conv.participants.len(), 2);
            assert_eq!(conv.created_by, user1);
            assert!(conv.name.is_none());
        }

        #[test]
        fn test_create_group_conversation() {
            let creator = Uuid::new_v4();
            let members = vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()];
            let conv = Conversation::new_group("Engineering Team".to_string(), creator, members.clone());
            
            assert!(matches!(conv.conversation_type, ConversationType::Group { .. }));
            assert_eq!(conv.participants.len(), 4); // creator + 3 members
            assert_eq!(conv.name, Some("Engineering Team".to_string()));
            
            // Creator should be first participant with Owner role
            assert_eq!(conv.participants[0].user_id, creator);
            assert_eq!(conv.participants[0].role, ParticipantRole::Owner);
        }
    }

    mod presence_tests {
        use crate::presence::{PresenceStatus, Status, TypingIndicator};
        use uuid::Uuid;

        #[test]
        fn test_presence_status() {
            let user_id = Uuid::new_v4();
            let presence = PresenceStatus::online(user_id);
            
            assert_eq!(presence.user_id, user_id);
            assert_eq!(presence.status, Status::Online);
        }

        #[test]
        fn test_typing_indicator_expiry() {
            let conv_id = Uuid::new_v4();
            let user_id = Uuid::new_v4();
            let indicator = TypingIndicator::new(conv_id, user_id);
            
            // Freshly created indicator should not be expired
            assert!(!indicator.is_expired());
        }
    }

    mod encryption_tests {
        use crate::encryption::{E2eeSession, KeyBundle};
        use uuid::Uuid;

        #[test]
        fn test_e2ee_session_creation() {
            let conv_id = Uuid::new_v4();
            let session = E2eeSession::new(conv_id);
            
            assert_eq!(session.conversation_id, conv_id);
            assert!(!session.initialized);
            assert!(!session.is_ready());
        }

        #[test]
        fn test_e2ee_session_initialization() {
            let conv_id = Uuid::new_v4();
            let mut session = E2eeSession::new(conv_id);
            
            let root_key = [1u8; 32];
            let chain_key = [2u8; 32];
            session.initialize(root_key, chain_key);
            
            assert!(session.initialized);
            assert!(session.is_ready());
        }

        #[test]
        fn test_key_bundle_creation() {
            let user_id = Uuid::new_v4();
            let bundle = KeyBundle::new(user_id);
            
            assert_eq!(bundle.user_id, user_id);
            assert_eq!(bundle.identity_key.len(), 32);
            assert!(bundle.one_time_prekeys.is_empty());
        }

        #[test]
        fn test_add_one_time_prekeys() {
            let user_id = Uuid::new_v4();
            let mut bundle = KeyBundle::new(user_id);
            bundle.add_one_time_prekeys(10);
            
            assert_eq!(bundle.one_time_prekeys.len(), 10);
            assert_eq!(bundle.one_time_prekeys[0].id, 1);
            assert_eq!(bundle.one_time_prekeys[9].id, 10);
        }
    }
}
