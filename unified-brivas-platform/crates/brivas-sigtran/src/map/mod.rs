//! MAP (Mobile Application Part) Layer
//!
//! GSM 09.02 / 3GPP TS 29.002 compliant SMS and USSD operations.

mod sms;
mod ussd;
mod encoding;

pub use sms::MapSmsOperation;
pub use ussd::MapUssdOperation;
pub use encoding::{encode_ussd_string, decode_ussd_string, encode_gsm7, decode_gsm7};

use crate::config::SigtranConfig;
use crate::errors::MapError;
use crate::sccp::{SccpAddress, SccpEndpoint, GlobalTitle};
use crate::tcap::{TcapEndpoint, TcapMessage, Component, DialoguePortion};
use crate::types::{SmRpDa, SmRpOa, RoutingInfo, UssdResponse, DataCodingScheme};
use std::sync::Arc;
use std::sync::atomic::{AtomicI32, Ordering};
use std::time::Duration;
use tokio::time::timeout;
use tracing::{debug, info, instrument};

/// MAP Operation Codes
pub mod operation {
    // Location Management
    pub const UPDATE_LOCATION: i32 = 2;
    pub const CANCEL_LOCATION: i32 = 3;
    
    // SMS Operations
    pub const SEND_ROUTING_INFO_FOR_SM: i32 = 45;
    pub const MO_FORWARD_SHORT_MESSAGE: i32 = 46;
    pub const MT_FORWARD_SHORT_MESSAGE: i32 = 44;
    pub const REPORT_SM_DELIVERY_STATUS: i32 = 47;
    
    // USSD Operations
    pub const PROCESS_UNSTRUCTURED_SS_REQUEST: i32 = 59;
    pub const UNSTRUCTURED_SS_REQUEST: i32 = 60;
    pub const UNSTRUCTURED_SS_NOTIFY: i32 = 61;
}

/// MAP Application Contexts
pub mod application_context {
    /// SMS Gateway Context v3
    pub const SHORT_MSG_GATEWAY_V3: &[u32] = &[0, 4, 0, 0, 1, 0, 20, 3];
    /// SMS Relay Context v3
    pub const SHORT_MSG_RELAY_V3: &[u32] = &[0, 4, 0, 0, 1, 0, 21, 3];
    /// USSD Network Initiated v2
    pub const NETWORK_USSD_V2: &[u32] = &[0, 4, 0, 0, 1, 0, 19, 2];
}

/// MAP Endpoint
pub struct MapEndpoint {
    /// TCAP endpoint
    tcap: Arc<TcapEndpoint>,
    /// SCCP endpoint
    sccp: Arc<SccpEndpoint>,
    /// Configuration
    config: SigtranConfig,
    /// Next invoke ID
    next_invoke_id: AtomicI32,
    /// Operation timeout
    timeout: Duration,
}

impl MapEndpoint {
    /// Create new MAP endpoint
    pub fn new(
        tcap: Arc<TcapEndpoint>,
        sccp: Arc<SccpEndpoint>,
        config: SigtranConfig,
    ) -> Self {
        let timeout = Duration::from_millis(config.map.operation_timeout_ms);
        
        Self {
            tcap,
            sccp,
            config,
            next_invoke_id: AtomicI32::new(1),
            timeout,
        }
    }

    /// Get next invoke ID
    fn next_invoke_id(&self) -> i32 {
        self.next_invoke_id.fetch_add(1, Ordering::Relaxed)
    }

    /// Create HLR address from config
    fn hlr_address(&self) -> SccpAddress {
        SccpAddress::from_gt(
            GlobalTitle::e164(&self.config.map.hlr_gt),
            Some(crate::ssn::HLR),
        )
    }

    /// Create MSC address from config
    fn msc_address(&self) -> SccpAddress {
        SccpAddress::from_gt(
            GlobalTitle::e164(&self.config.map.msc_gt),
            Some(crate::ssn::MSC),
        )
    }

    // ==================== SMS Operations ====================

    /// Send Routing Info for SM (SRI-SM)
    /// Used to query HLR for routing information before MT-SMS
    #[instrument(skip(self))]
    pub async fn send_routing_info_for_sm(
        &self,
        msisdn: &str,
    ) -> Result<RoutingInfo, MapError> {
        info!("Sending SRI-SM for {}", msisdn);
        
        let invoke_id = self.next_invoke_id();
        
        // Build SRI-SM parameter
        let param = sms::encode_sri_sm_request(
            msisdn,
            &self.config.map.service_centre_address,
            true, // SM-RP-PRI
        );
        
        let component = Component::Invoke {
            invoke_id,
            linked_id: None,
            operation_code: operation::SEND_ROUTING_INFO_FOR_SM,
            parameter: Some(param),
        };
        
        let tid = self.tcap.begin(
            &self.hlr_address(),
            application_context::SHORT_MSG_GATEWAY_V3.to_vec(),
            vec![component],
        ).await?;
        
        // Wait for response
        let response = timeout(self.timeout, self.tcap.receive()).await
            .map_err(|_| MapError::Tcap(crate::errors::TcapError::InvalidState("Timeout".to_string())))??;
        
        match response {
            TcapMessage::End { component_portion, .. } |
            TcapMessage::Continue { component_portion, .. } => {
                for comp in component_portion {
                    match comp {
                        Component::ReturnResultLast { parameter, .. } => {
                            if let Some(param) = parameter {
                                return sms::decode_sri_sm_response(&param);
                            }
                        }
                        Component::ReturnError { error_code, .. } => {
                            return Err(MapError::OperationError {
                                code: error_code,
                                parameter: None,
                            });
                        }
                        _ => {}
                    }
                }
                Err(MapError::SystemFailure)
            }
            _ => Err(MapError::SystemFailure),
        }
    }

    /// Mobile Originated Forward Short Message (MO-FSM)
    #[instrument(skip(self, tpdu))]
    pub async fn mo_forward_sm(
        &self,
        destination: &str,
        originator: &str,
        tpdu: &[u8],
    ) -> Result<(), MapError> {
        info!("Sending MO-FSM from {} to {}", originator, destination);
        
        let invoke_id = self.next_invoke_id();
        
        let param = sms::encode_mo_forward_sm(
            SmRpDa::ServiceCentreAddress(self.config.map.service_centre_address.clone()),
            SmRpOa::Msisdn(originator.to_string()),
            tpdu,
        );
        
        let component = Component::Invoke {
            invoke_id,
            linked_id: None,
            operation_code: operation::MO_FORWARD_SHORT_MESSAGE,
            parameter: Some(param),
        };
        
        let tid = self.tcap.begin(
            &self.msc_address(),
            application_context::SHORT_MSG_RELAY_V3.to_vec(),
            vec![component],
        ).await?;
        
        // Wait for acknowledgement
        let response = timeout(self.timeout, self.tcap.receive()).await
            .map_err(|_| MapError::Tcap(crate::errors::TcapError::InvalidState("Timeout".to_string())))??;
        
        match response {
            TcapMessage::End { component_portion, .. } => {
                for comp in component_portion {
                    if let Component::ReturnError { error_code, .. } = comp {
                        return Err(MapError::OperationError {
                            code: error_code,
                            parameter: None,
                        });
                    }
                }
                Ok(())
            }
            _ => Err(MapError::SystemFailure),
        }
    }

    /// Mobile Terminated Forward Short Message (MT-FSM)
    #[instrument(skip(self, tpdu))]
    pub async fn mt_forward_sm(
        &self,
        msisdn: &str,
        service_centre: &str,
        tpdu: &[u8],
    ) -> Result<(), MapError> {
        info!("Sending MT-FSM to {}", msisdn);
        
        // First get routing info
        let routing_info = self.send_routing_info_for_sm(msisdn).await?;
        
        let invoke_id = self.next_invoke_id();
        
        let param = sms::encode_mt_forward_sm(
            SmRpDa::Imsi(routing_info.imsi),
            SmRpOa::ServiceCentreAddress(service_centre.to_string()),
            tpdu,
            false, // more_messages_to_send
        );
        
        let component = Component::Invoke {
            invoke_id,
            linked_id: None,
            operation_code: operation::MT_FORWARD_SHORT_MESSAGE,
            parameter: Some(param),
        };
        
        // Send to MSC from routing info
        let msc_address = SccpAddress::from_gt(
            GlobalTitle::e164(&routing_info.msc_number),
            Some(crate::ssn::MSC),
        );
        
        let tid = self.tcap.begin(
            &msc_address,
            application_context::SHORT_MSG_RELAY_V3.to_vec(),
            vec![component],
        ).await?;
        
        let response = timeout(self.timeout, self.tcap.receive()).await
            .map_err(|_| MapError::Tcap(crate::errors::TcapError::InvalidState("Timeout".to_string())))??;
        
        match response {
            TcapMessage::End { component_portion, .. } => {
                for comp in component_portion {
                    if let Component::ReturnError { error_code, .. } = comp {
                        return Err(MapError::OperationError {
                            code: error_code,
                            parameter: None,
                        });
                    }
                }
                Ok(())
            }
            _ => Err(MapError::SystemFailure),
        }
    }

    // ==================== USSD Operations ====================

    /// Process USSD Request (network-initiated)
    #[instrument(skip(self))]
    pub async fn process_ussd(
        &self,
        msisdn: &str,
        ussd_string: &str,
        dcs: u8,
    ) -> Result<UssdResponse, MapError> {
        info!("Processing USSD for {}: {}", msisdn, ussd_string);
        
        let invoke_id = self.next_invoke_id();
        
        let encoded_string = encode_ussd_string(ussd_string, dcs)?;
        let param = ussd::encode_process_ussd_request(dcs, &encoded_string, Some(msisdn));
        
        let component = Component::Invoke {
            invoke_id,
            linked_id: None,
            operation_code: operation::PROCESS_UNSTRUCTURED_SS_REQUEST,
            parameter: Some(param),
        };
        
        let hlr = self.hlr_address();
        
        let tid = self.tcap.begin(
            &hlr,
            application_context::NETWORK_USSD_V2.to_vec(),
            vec![component],
        ).await?;
        
        let response = timeout(self.timeout, self.tcap.receive()).await
            .map_err(|_| MapError::Tcap(crate::errors::TcapError::InvalidState("Timeout".to_string())))??;
        
        match response {
            TcapMessage::End { component_portion, .. } |
            TcapMessage::Continue { component_portion, .. } => {
                for comp in component_portion {
                    match comp {
                        Component::ReturnResultLast { parameter, .. } |
                        Component::ReturnResultNotLast { parameter, .. } => {
                            if let Some(param) = parameter {
                                return ussd::decode_ussd_response(&param);
                            }
                        }
                        Component::ReturnError { error_code, .. } => {
                            return Err(MapError::OperationError {
                                code: error_code,
                                parameter: None,
                            });
                        }
                        _ => {}
                    }
                }
                Err(MapError::SystemFailure)
            }
            _ => Err(MapError::SystemFailure),
        }
    }

    /// Send USSD Request (mobile-initiated)
    #[instrument(skip(self))]
    pub async fn send_ussd(
        &self,
        msisdn: &str,
        ussd_string: &str,
        dcs: u8,
    ) -> Result<(), MapError> {
        info!("Sending USSD to {}: {}", msisdn, ussd_string);
        
        let invoke_id = self.next_invoke_id();
        
        let encoded_string = encode_ussd_string(ussd_string, dcs)?;
        let param = ussd::encode_ussd_request(dcs, &encoded_string, msisdn);
        
        let component = Component::Invoke {
            invoke_id,
            linked_id: None,
            operation_code: operation::UNSTRUCTURED_SS_REQUEST,
            parameter: Some(param),
        };
        
        let msc = self.msc_address();
        
        self.tcap.begin(
            &msc,
            application_context::NETWORK_USSD_V2.to_vec(),
            vec![component],
        ).await?;
        
        Ok(())
    }
}
