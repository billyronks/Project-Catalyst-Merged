//! Unit tests for Flash Call Service

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_otp_generation() {
        // Test OTP generation produces correct length
        let otp4 = FlashCallService::generate_otp(4);
        assert_eq!(otp4.len(), 4);
        assert!(otp4.chars().all(|c| c.is_ascii_digit()));

        let otp6 = FlashCallService::generate_otp(6);
        assert_eq!(otp6.len(), 6);
        assert!(otp6.chars().all(|c| c.is_ascii_digit()));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        use crate::VoiceIvrConfig;
        
        let config = VoiceIvrConfig {
            pop_id: "test".to_string(),
            lumadb_url: "postgres://test:test@localhost/test".to_string(),
            opensips_url: "http://localhost:8080".to_string(),
            freeswitch_url: "localhost:8021".to_string(),
            freeswitch_password: "test".to_string(),
            rtpengine_url: "udp:127.0.0.1:22222".to_string(),
            stir_shaken_enabled: false,
            stir_shaken_cert_path: None,
            stir_shaken_key_path: None,
        };
        
        let service = FlashCallService::new(&config).await.unwrap();
        
        // First 5 requests should succeed
        for i in 0..5 {
            let result = service.check_rate_limit("+1234567890");
            assert!(result.is_ok(), "Request {} should succeed", i);
        }
        
        // 6th request should be rate limited
        let result = service.check_rate_limit("+1234567890");
        assert!(result.is_err(), "6th request should be rate limited");
    }

    #[tokio::test]
    async fn test_flash_call_flow() {
        use crate::VoiceIvrConfig;
        
        let config = VoiceIvrConfig {
            pop_id: "test".to_string(),
            lumadb_url: "postgres://test:test@localhost/test".to_string(),
            opensips_url: "http://localhost:8080".to_string(),
            freeswitch_url: "localhost:8021".to_string(),
            freeswitch_password: "test".to_string(),
            rtpengine_url: "udp:127.0.0.1:22222".to_string(),
            stir_shaken_enabled: false,
            stir_shaken_cert_path: None,
            stir_shaken_key_path: None,
        };
        
        let service = FlashCallService::new(&config).await.unwrap();
        
        // Initiate flash call
        let request = FlashCallRequest {
            request_id: "test-123".to_string(),
            destination: "+1234567890".to_string(),
            cli_prefix: "1".to_string(),
            otp_length: 4,
            callback_url: None,
            metadata: None,
        };
        
        let response = service.initiate(request).await.unwrap();
        assert_eq!(response.request_id, "test-123");
        assert_eq!(response.otp.len(), 4);
        assert_eq!(response.status, FlashCallStatus::Initiated);
        
        // Verify with correct OTP
        let verify_result = service.verify_otp("test-123", &response.otp).await.unwrap();
        assert_eq!(verify_result, VerificationResult::Success);
        
        // Verify again should fail (already used)
        let verify_again = service.verify_otp("test-123", &response.otp).await.unwrap();
        assert_eq!(verify_again, VerificationResult::AlreadyUsed);
    }

    #[tokio::test]
    async fn test_invalid_otp() {
        use crate::VoiceIvrConfig;
        
        let config = VoiceIvrConfig {
            pop_id: "test".to_string(),
            lumadb_url: "postgres://test:test@localhost/test".to_string(),
            opensips_url: "http://localhost:8080".to_string(),
            freeswitch_url: "localhost:8021".to_string(),
            freeswitch_password: "test".to_string(),
            rtpengine_url: "udp:127.0.0.1:22222".to_string(),
            stir_shaken_enabled: false,
            stir_shaken_cert_path: None,
            stir_shaken_key_path: None,
        };
        
        let service = FlashCallService::new(&config).await.unwrap();
        
        let request = FlashCallRequest {
            request_id: "test-456".to_string(),
            destination: "+1987654321".to_string(),
            cli_prefix: "1".to_string(),
            otp_length: 4,
            callback_url: None,
            metadata: None,
        };
        
        service.initiate(request).await.unwrap();
        
        // Verify with wrong OTP
        let verify_result = service.verify_otp("test-456", "0000").await.unwrap();
        assert_eq!(verify_result, VerificationResult::InvalidOtp);
    }
}
