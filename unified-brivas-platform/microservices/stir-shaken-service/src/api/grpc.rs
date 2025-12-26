//! gRPC API implementation for STIR/SHAKEN service

use tonic::{Request, Response, Status};
use crate::proto::stir_shaken_service_server::StirShakenService;
use crate::proto::*;
use crate::certificate::CertificateManager;
use crate::attestation::AttestationSigner;
use crate::verification::VerificationService;

pub struct StirShakenGrpcService {
    cert_manager: CertificateManager,
    signer: AttestationSigner,
    verifier: VerificationService,
}

impl StirShakenGrpcService {
    pub fn new(
        cert_manager: CertificateManager,
        signer: AttestationSigner,
        verifier: VerificationService,
    ) -> Self {
        Self {
            cert_manager,
            signer,
            verifier,
        }
    }
}

#[tonic::async_trait]
impl StirShakenService for StirShakenGrpcService {
    /// Sign a call and generate PASSporT
    async fn sign_call(
        &self,
        request: Request<SignCallRequest>,
    ) -> Result<Response<SignCallResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!(
            orig = %req.orig_tn,
            dest = %req.dest_tn,
            "Signing call"
        );

        let start = std::time::Instant::now();
        
        match self.signer.sign(&req).await {
            Ok(response) => {
                let latency = start.elapsed();
                metrics::histogram!("stir_shaken_sign_latency_ms", latency.as_millis() as f64);
                metrics::counter!("stir_shaken_signs_total", 1);
                Ok(Response::new(response))
            }
            Err(e) => {
                metrics::counter!("stir_shaken_sign_errors_total", 1);
                Err(Status::internal(format!("Signing failed: {}", e)))
            }
        }
    }

    /// Batch sign multiple calls
    async fn batch_sign_calls(
        &self,
        request: Request<BatchSignCallsRequest>,
    ) -> Result<Response<BatchSignCallsResponse>, Status> {
        let req = request.into_inner();
        let mut responses = Vec::with_capacity(req.requests.len());
        let mut success_count = 0i32;
        let mut failure_count = 0i32;
        let mut errors = Vec::new();

        for (idx, sign_req) in req.requests.into_iter().enumerate() {
            match self.signer.sign(&sign_req).await {
                Ok(resp) => {
                    responses.push(resp);
                    success_count += 1;
                }
                Err(e) => {
                    failure_count += 1;
                    errors.push(SignError {
                        index: idx as i32,
                        error_code: "SIGN_FAILED".to_string(),
                        error_message: e.to_string(),
                    });
                }
            }
        }

        Ok(Response::new(BatchSignCallsResponse {
            responses,
            success_count,
            failure_count,
            errors,
        }))
    }

    /// Get attestation level for a caller/callee pair
    async fn get_attestation_level(
        &self,
        request: Request<GetAttestationLevelRequest>,
    ) -> Result<Response<GetAttestationLevelResponse>, Status> {
        let req = request.into_inner();
        
        let (level, tn_authorized) = self.signer
            .determine_attestation_level(&req.caller_tn, &req.customer_id)
            .await;

        Ok(Response::new(GetAttestationLevelResponse {
            level: level as i32,
            reason: if tn_authorized { 
                "TN authorized for this customer".to_string() 
            } else { 
                "Gateway attestation - TN not authorized".to_string() 
            },
            tn_authorized,
        }))
    }

    /// Verify a PASSporT from Identity header
    async fn verify_call(
        &self,
        request: Request<VerifyCallRequest>,
    ) -> Result<Response<VerifyCallResponse>, Status> {
        let req = request.into_inner();
        
        tracing::debug!(
            from = %req.from_tn,
            to = %req.to_tn,
            "Verifying call"
        );

        let start = std::time::Instant::now();
        
        match self.verifier.verify(&req).await {
            Ok(response) => {
                let latency = start.elapsed();
                metrics::histogram!("stir_shaken_verify_latency_ms", latency.as_millis() as f64);
                metrics::counter!("stir_shaken_verifications_total", 1);
                Ok(Response::new(response))
            }
            Err(e) => {
                metrics::counter!("stir_shaken_verify_errors_total", 1);
                Err(Status::internal(format!("Verification failed: {}", e)))
            }
        }
    }

    /// Batch verify multiple calls
    async fn batch_verify_calls(
        &self,
        request: Request<BatchVerifyCallsRequest>,
    ) -> Result<Response<BatchVerifyCallsResponse>, Status> {
        let req = request.into_inner();
        let mut responses = Vec::with_capacity(req.requests.len());
        let mut valid_count = 0i32;
        let mut invalid_count = 0i32;

        for verify_req in req.requests {
            match self.verifier.verify(&verify_req).await {
                Ok(resp) => {
                    if resp.status == VerificationStatus::VerificationValid as i32 {
                        valid_count += 1;
                    } else {
                        invalid_count += 1;
                    }
                    responses.push(resp);
                }
                Err(_) => {
                    invalid_count += 1;
                    responses.push(VerifyCallResponse {
                        status: VerificationStatus::VerificationUnknown as i32,
                        ..Default::default()
                    });
                }
            }
        }

        Ok(Response::new(BatchVerifyCallsResponse {
            responses,
            valid_count,
            invalid_count,
        }))
    }

    /// List all certificates
    async fn list_certificates(
        &self,
        request: Request<ListCertificatesRequest>,
    ) -> Result<Response<ListCertificatesResponse>, Status> {
        let req = request.into_inner();
        let certs = self.cert_manager.list_certificates(
            req.include_expired,
            req.include_revoked,
            req.pop_id.as_str(),
        ).await;

        Ok(Response::new(ListCertificatesResponse { certificates: certs }))
    }

    /// Get certificate details
    async fn get_certificate(
        &self,
        request: Request<GetCertificateRequest>,
    ) -> Result<Response<Certificate>, Status> {
        let req = request.into_inner();
        
        match self.cert_manager.get_certificate(&req.id).await {
            Some(cert) => Ok(Response::new(cert)),
            None => Err(Status::not_found("Certificate not found")),
        }
    }

    /// Upload a new certificate
    async fn upload_certificate(
        &self,
        request: Request<UploadCertificateRequest>,
    ) -> Result<Response<Certificate>, Status> {
        let req = request.into_inner();
        
        match self.cert_manager.upload_certificate(req).await {
            Ok(cert) => Ok(Response::new(cert)),
            Err(e) => Err(Status::invalid_argument(format!("Upload failed: {}", e))),
        }
    }

    /// Delete a certificate
    async fn delete_certificate(
        &self,
        request: Request<DeleteCertificateRequest>,
    ) -> Result<Response<()>, Status> {
        let req = request.into_inner();
        
        match self.cert_manager.delete_certificate(&req.id).await {
            Ok(_) => Ok(Response::new(())),
            Err(e) => Err(Status::internal(format!("Delete failed: {}", e))),
        }
    }

    /// Trigger certificate rotation
    async fn rotate_certificate(
        &self,
        request: Request<RotateCertificateRequest>,
    ) -> Result<Response<Certificate>, Status> {
        let req = request.into_inner();
        
        match self.cert_manager.rotate_certificate(&req.id, req.use_acme).await {
            Ok(cert) => Ok(Response::new(cert)),
            Err(e) => Err(Status::internal(format!("Rotation failed: {}", e))),
        }
    }

    /// Register telephone numbers for attestation
    async fn register_telephone_numbers(
        &self,
        request: Request<RegisterTelephoneNumbersRequest>,
    ) -> Result<Response<RegisterTelephoneNumbersResponse>, Status> {
        let req = request.into_inner();
        
        match self.signer.register_tns(req.numbers).await {
            Ok((registered, failed, errors)) => {
                Ok(Response::new(RegisterTelephoneNumbersResponse {
                    registered_count: registered,
                    failed_count: failed,
                    errors,
                }))
            }
            Err(e) => Err(Status::internal(format!("Registration failed: {}", e))),
        }
    }

    /// Check if we can attest for a number
    async fn check_tn_authorization(
        &self,
        request: Request<CheckTnAuthorizationRequest>,
    ) -> Result<Response<CheckTnAuthorizationResponse>, Status> {
        let req = request.into_inner();
        
        let (level, authorized) = self.signer
            .determine_attestation_level(&req.tn, &req.customer_id)
            .await;

        Ok(Response::new(CheckTnAuthorizationResponse {
            authorized,
            max_attestation: level as i32,
            reason: if authorized {
                "TN authorized".to_string()
            } else {
                "TN not found in authorization list".to_string()
            },
        }))
    }

    /// Health check
    async fn health_check(
        &self,
        _request: Request<()>,
    ) -> Result<Response<HealthCheckResponse>, Status> {
        Ok(Response::new(HealthCheckResponse {
            healthy: true,
            version: env!("CARGO_PKG_VERSION").to_string(),
            uptime_since: None,
            lumadb_healthy: true,
            hsm_healthy: true,
            certificates_valid: self.cert_manager.has_active_certificate(),
        }))
    }

    /// Get service statistics
    async fn get_statistics(
        &self,
        _request: Request<()>,
    ) -> Result<Response<StatisticsResponse>, Status> {
        let stats = self.signer.get_statistics().await;
        Ok(Response::new(stats))
    }
}
