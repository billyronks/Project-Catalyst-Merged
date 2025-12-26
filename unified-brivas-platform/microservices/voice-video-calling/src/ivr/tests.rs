//! Unit tests for IVR Engine

#[cfg(test)]
mod tests {
    use crate::ivr::flow::IvrFlow;
    use crate::ivr::nodes::IvrNode;
    use crate::ivr::engine::IvrEngine;
    use crate::VoiceIvrConfig;
    use std::collections::HashMap;

    #[test]
    fn test_ivr_flow_creation() {
        let flow = IvrFlow::new("test-flow", "start");
        assert_eq!(flow.name, "test-flow");
        assert_eq!(flow.entry_node, "start");
        assert!(!flow.id.is_empty());
    }

    #[test]
    fn test_ivr_flow_add_node() {
        let mut flow = IvrFlow::new("test", "start");
        
        let node = IvrNode::play_audio("start", "/audio/welcome.wav", "next");
        flow.add_node(node);
        
        assert_eq!(flow.nodes.len(), 1);
        assert!(flow.nodes.contains_key("start"));
    }

    #[test]
    fn test_ivr_flow_validation_empty() {
        let flow = IvrFlow::new("test", "start");
        let result = flow.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_ivr_flow_validation_missing_entry() {
        let mut flow = IvrFlow::new("test", "start");
        flow.add_node(IvrNode::hangup("other", "normal"));
        
        let result = flow.validate();
        assert!(result.is_err());
    }

    #[test]
    fn test_ivr_flow_validation_success() {
        let mut flow = IvrFlow::new("test", "start");
        // Use hangup as terminal node (no dangling references)
        flow.add_node(IvrNode::hangup("start", "normal"));
        
        let result = flow.validate();
        assert!(result.is_ok());
    }

    #[test]
    fn test_ivr_node_menu() {
        let mut branches = HashMap::new();
        branches.insert("1".to_string(), "option1".to_string());
        branches.insert("2".to_string(), "option2".to_string());
        
        let node = IvrNode::menu("menu", "Press 1 or 2", branches);
        
        let next_nodes = node.get_next_nodes();
        assert!(next_nodes.contains(&"option1".to_string()));
        assert!(next_nodes.contains(&"option2".to_string()));
    }

    #[tokio::test]
    async fn test_ivr_engine_flow_management() {
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
        
        let engine = IvrEngine::new(&config).await.unwrap();
        
        // Create and register a flow
        let mut flow = IvrFlow::new("welcome", "start");
        flow.add_node(IvrNode::hangup("start", "normal"));
        
        let flow_id = engine.create_flow(flow).unwrap();
        assert!(!flow_id.is_empty());
        
        // List flows
        let flows = engine.list_flows();
        assert_eq!(flows.len(), 1);
        
        // Get flow
        let retrieved = engine.get_flow(&flow_id);
        assert!(retrieved.is_some());
    }

    #[tokio::test]
    async fn test_ivr_session_creation() {
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
        
        let engine = IvrEngine::new(&config).await.unwrap();
        
        // Create flow
        let mut flow = IvrFlow::new("test", "start");
        flow.add_node(IvrNode::hangup("start", "normal"));
        let flow_id = engine.create_flow(flow).unwrap();
        
        // Start session - returns session_id string
        let session_id = engine.start_session("call-123", &flow_id).await.unwrap();
        assert!(!session_id.is_empty());
    }
}
