//! Tests for brivas-rcs-sdk

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    mod agent_tests {
        use crate::agent::{RcsAgent, AgentCategory, AgentVerificationStatus};
        use uuid::Uuid;

        #[test]
        fn test_create_agent() {
            let tenant_id = Uuid::new_v4();
            let agent = RcsAgent::new(
                tenant_id,
                "test-agent".to_string(),
                "Test Brand".to_string(),
                "https://cdn.brivas.io/logo.png".to_string(),
                "#FF5722".to_string(),
                "https://api.example.com/webhook".to_string(),
            );

            assert_eq!(agent.tenant_id, tenant_id);
            assert_eq!(agent.name, "test-agent");
            assert_eq!(agent.verification_status, AgentVerificationStatus::Pending);
            assert_eq!(agent.category, AgentCategory::Transactional);
        }
    }

    mod message_tests {
        use crate::message::{RcsMessage, RcsMessageStatus, RcsMessageType};
        use uuid::Uuid;

        #[test]
        fn test_create_text_message() {
            let agent_id = Uuid::new_v4();
            let message = RcsMessage::new_text(
                agent_id,
                "+2348012345678".to_string(),
                "Hello from RCS!".to_string(),
            );

            assert_eq!(message.agent_id, agent_id);
            assert_eq!(message.recipient_phone, "+2348012345678");
            assert_eq!(message.message_type, RcsMessageType::Text);
            assert_eq!(message.status, RcsMessageStatus::Pending);
            assert!(!message.fallback_to_sms);
        }
    }

    mod rich_card_tests {
        use crate::rich_card::{
            RichCard, StandaloneCard, CardContent, CardOrientation, Media, MediaHeight,
        };

        #[test]
        fn test_create_standalone_card() {
            let content = CardContent::new("Product Title", "This is a great product!");
            let card = StandaloneCard::vertical(content);

            assert_eq!(card.card_orientation, CardOrientation::Vertical);
            assert!(card.thumbnail_image_alignment.is_none());
        }

        #[test]
        fn test_card_with_media() {
            let media = Media::from_url("https://cdn.brivas.io/image.jpg", MediaHeight::Medium);
            let content = CardContent::new("Title", "Description").with_media(media);

            assert!(content.media.is_some());
            assert_eq!(content.media.unwrap().height, MediaHeight::Medium);
        }

        #[test]
        fn test_rich_card_container() {
            let content = CardContent::new("Title", "Description");
            let standalone = StandaloneCard::vertical(content);
            let rich_card = RichCard::standalone(standalone);

            assert!(rich_card.standalone_card.is_some());
            assert!(rich_card.carousel_card.is_none());
        }
    }

    mod carousel_tests {
        use crate::carousel::{CarouselCard, CardWidth};
        use crate::rich_card::CardContent;

        #[test]
        fn test_create_carousel() {
            let cards = vec![
                CardContent::new("Card 1", "First card"),
                CardContent::new("Card 2", "Second card"),
                CardContent::new("Card 3", "Third card"),
            ];
            let carousel = CarouselCard::new(cards);

            assert_eq!(carousel.card_width, CardWidth::Medium);
            assert_eq!(carousel.card_contents.len(), 3);
        }

        #[test]
        fn test_carousel_add_card() {
            let initial_cards = vec![
                CardContent::new("Card 1", "First"),
                CardContent::new("Card 2", "Second"),
            ];
            let mut carousel = CarouselCard::new(initial_cards);
            
            carousel.add_card(CardContent::new("Card 3", "Third")).unwrap();
            assert_eq!(carousel.card_contents.len(), 3);
        }
    }

    mod suggestion_tests {
        use crate::suggestion::Suggestion;

        #[test]
        fn test_suggested_reply() {
            let suggestion = Suggestion::reply("Yes", "postback_yes");

            assert!(suggestion.reply.is_some());
            assert!(suggestion.action.is_none());
            assert_eq!(suggestion.reply.as_ref().unwrap().text, "Yes");
        }

        #[test]
        fn test_open_url_action() {
            let suggestion = Suggestion::open_url("Visit Website", "https://brivas.io");

            assert!(suggestion.action.is_some());
            assert!(suggestion.reply.is_none());
        }

        #[test]
        fn test_dial_action() {
            let suggestion = Suggestion::dial("Call Us", "+2348012345678");

            assert!(suggestion.action.is_some());
        }

        #[test]
        fn test_location_action() {
            let suggestion = Suggestion::location("View Location", 6.5244, 3.3792, Some("Lagos".to_string()));

            assert!(suggestion.action.is_some());
        }
    }

    mod capability_tests {
        use crate::capability::{DeviceCapability, RcsFeatures, RcsHub, MessageChannel};
        use chrono::{Duration, Utc};

        #[test]
        fn test_device_capability_check() {
            let now = Utc::now();
            let capability = DeviceCapability {
                phone_number: "+2348012345678".to_string(),
                rcs_enabled: true,
                carrier: Some("MTN Nigeria".to_string()),
                carrier_rcs_hub: Some(RcsHub::GoogleJibe),
                features: RcsFeatures::full(),
                checked_at: now,
                cache_valid_until: now + Duration::hours(24),
            };

            assert!(capability.is_valid());
            assert!(matches!(capability.to_channel(), MessageChannel::Rcs { .. }));
        }

        #[test]
        fn test_sms_fallback_channel() {
            let now = Utc::now();
            let capability = DeviceCapability {
                phone_number: "+18001234567".to_string(),
                rcs_enabled: false,
                carrier: None,
                carrier_rcs_hub: None,
                features: RcsFeatures::default(),
                checked_at: now,
                cache_valid_until: now + Duration::hours(24),
            };

            assert!(matches!(capability.to_channel(), MessageChannel::SmsFallback));
        }

        #[test]
        fn test_rcs_features() {
            let full = RcsFeatures::full();
            let basic = RcsFeatures::basic();

            assert!(full.carousel);
            assert!(!basic.carousel);
            assert_eq!(full.file_transfer_max_size_mb, 100);
            assert_eq!(basic.file_transfer_max_size_mb, 10);
        }
    }
}
