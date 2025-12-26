//! Invoice Service
//!
//! Generates and manages invoices for postpaid customers.

use chrono::{DateTime, Datelike, Duration, Utc};
use dashmap::DashMap;
use rust_decimal::Decimal;
use std::sync::Arc;
use uuid::Uuid;

use crate::types::{Invoice, InvoiceLineItem, InvoiceStatus, ServiceType};

#[derive(Clone)]
pub struct InvoiceService {
    /// Invoice storage
    invoices: Arc<DashMap<Uuid, Invoice>>,
    /// Default currency
    default_currency: String,
    /// LumaDB URL
    #[allow(dead_code)]
    lumadb_url: String,
}

impl InvoiceService {
    pub async fn new(lumadb_url: &str, default_currency: &str) -> brivas_core::Result<Self> {
        Ok(Self {
            invoices: Arc::new(DashMap::new()),
            default_currency: default_currency.to_string(),
            lumadb_url: lumadb_url.to_string(),
        })
    }

    /// Generate invoice number
    fn generate_invoice_number(&self, customer_id: Uuid) -> String {
        let now = Utc::now();
        format!(
            "INV-{}-{:04}{:02}-{:04}",
            &customer_id.to_string()[..8],
            now.year(),
            now.month(),
            self.invoices.len() + 1
        )
    }

    /// Create a new invoice
    pub async fn create_invoice(
        &self,
        customer_id: Uuid,
        billing_period_start: DateTime<Utc>,
        billing_period_end: DateTime<Utc>,
        line_items: Vec<InvoiceLineItem>,
    ) -> brivas_core::Result<Invoice> {
        let subtotal = line_items.iter().map(|li| li.amount).sum();
        let tax_rate = Decimal::new(75, 3); // 7.5% VAT
        let tax_amount = subtotal * tax_rate;
        let total_amount = subtotal + tax_amount;

        let invoice = Invoice {
            id: Uuid::new_v4(),
            customer_id,
            invoice_number: self.generate_invoice_number(customer_id),
            billing_period_start,
            billing_period_end,
            subtotal,
            tax_amount,
            total_amount,
            currency: self.default_currency.clone(),
            status: InvoiceStatus::Draft,
            due_date: Utc::now() + Duration::days(30),
            paid_at: None,
            created_at: Utc::now(),
            line_items,
        };

        self.invoices.insert(invoice.id, invoice.clone());
        Ok(invoice)
    }

    /// Generate invoice from CDRs for a customer
    pub async fn generate_from_cdrs(
        &self,
        customer_id: Uuid,
        cdrs: &[crate::types::Cdr],
    ) -> brivas_core::Result<Invoice> {
        // Group CDRs by service type
        let mut by_service: std::collections::HashMap<ServiceType, Vec<&crate::types::Cdr>> =
            std::collections::HashMap::new();
        
        for cdr in cdrs {
            by_service.entry(cdr.service_type).or_default().push(cdr);
        }

        let mut line_items = Vec::new();
        for (service_type, service_cdrs) in by_service {
            let quantity = service_cdrs.len() as u32;
            let amount: Decimal = service_cdrs
                .iter()
                .filter_map(|c| c.rated_amount)
                .sum();
            let unit_price = if quantity > 0 {
                amount / Decimal::from(quantity)
            } else {
                Decimal::ZERO
            };

            line_items.push(InvoiceLineItem {
                id: Uuid::new_v4(),
                description: format!("{:?} Messages", service_type),
                service_type,
                quantity,
                unit_price,
                amount,
            });
        }

        let period_start = cdrs.iter().map(|c| c.start_time).min().unwrap_or_else(Utc::now);
        let period_end = cdrs.iter().map(|c| c.start_time).max().unwrap_or_else(Utc::now);

        self.create_invoice(customer_id, period_start, period_end, line_items).await
    }

    /// Get invoice by ID
    pub async fn get_invoice(&self, id: Uuid) -> Option<Invoice> {
        self.invoices.get(&id).map(|i| i.clone())
    }

    /// List invoices for customer
    pub async fn list_invoices(&self, customer_id: Uuid) -> Vec<Invoice> {
        self.invoices
            .iter()
            .filter(|i| i.value().customer_id == customer_id)
            .map(|i| i.value().clone())
            .collect()
    }

    /// Mark invoice as paid
    pub async fn mark_paid(&self, invoice_id: Uuid) -> brivas_core::Result<()> {
        if let Some(mut invoice) = self.invoices.get_mut(&invoice_id) {
            invoice.status = InvoiceStatus::Paid;
            invoice.paid_at = Some(Utc::now());
        }
        Ok(())
    }

    /// Send invoice to customer
    pub async fn send_invoice(&self, invoice_id: Uuid) -> brivas_core::Result<()> {
        if let Some(mut invoice) = self.invoices.get_mut(&invoice_id) {
            invoice.status = InvoiceStatus::Sent;
        }
        // TODO: Send email/notification
        Ok(())
    }
}
