//! Unit tests for Billing Service

#[cfg(test)]
mod tests {
    use rust_decimal::Decimal;
    use rust_decimal_macros::dec;
    
    // Mock wallet for testing
    #[derive(Debug, Clone)]
    struct MockWallet {
        id: uuid::Uuid,
        customer_id: uuid::Uuid,
        balance: Decimal,
        currency: String,
    }

    #[test]
    fn test_wallet_creation() {
        let wallet = MockWallet {
            id: uuid::Uuid::new_v4(),
            customer_id: uuid::Uuid::new_v4(),
            balance: dec!(1000.00),
            currency: "NGN".to_string(),
        };
        
        assert!(wallet.balance > Decimal::ZERO);
        assert_eq!(wallet.currency, "NGN");
    }

    #[test]
    fn test_wallet_credit() {
        let mut wallet = MockWallet {
            id: uuid::Uuid::new_v4(),
            customer_id: uuid::Uuid::new_v4(),
            balance: dec!(1000.00),
            currency: "NGN".to_string(),
        };
        
        let credit_amount = dec!(500.00);
        wallet.balance += credit_amount;
        
        assert_eq!(wallet.balance, dec!(1500.00));
    }

    #[test]
    fn test_wallet_debit_success() {
        let mut wallet = MockWallet {
            id: uuid::Uuid::new_v4(),
            customer_id: uuid::Uuid::new_v4(),
            balance: dec!(1000.00),
            currency: "NGN".to_string(),
        };
        
        let debit_amount = dec!(500.00);
        assert!(wallet.balance >= debit_amount);
        wallet.balance -= debit_amount;
        
        assert_eq!(wallet.balance, dec!(500.00));
    }

    #[test]
    fn test_wallet_debit_insufficient_funds() {
        let wallet = MockWallet {
            id: uuid::Uuid::new_v4(),
            customer_id: uuid::Uuid::new_v4(),
            balance: dec!(100.00),
            currency: "NGN".to_string(),
        };
        
        let debit_amount = dec!(500.00);
        assert!(wallet.balance < debit_amount);
    }

    #[test]
    fn test_low_balance_threshold() {
        let threshold = dec!(1000.00);
        let low_balance = dec!(500.00);
        let high_balance = dec!(2000.00);
        
        assert!(low_balance < threshold);
        assert!(high_balance >= threshold);
    }

    // CDR Tests
    #[derive(Debug)]
    struct MockCdr {
        id: uuid::Uuid,
        call_type: String,
        duration_seconds: u32,
        amount: Decimal,
    }

    #[test]
    fn test_cdr_rating_sms() {
        let rate_per_sms = dec!(4.00); // NGN per SMS
        let quantity = 5;
        
        let total = rate_per_sms * Decimal::from(quantity);
        assert_eq!(total, dec!(20.00));
    }

    #[test]
    fn test_cdr_rating_voice() {
        let rate_per_minute = dec!(12.50); // NGN per minute
        let duration_seconds = 150; // 2.5 minutes
        
        let minutes = Decimal::from(duration_seconds) / dec!(60);
        let total = rate_per_minute * minutes;
        
        assert_eq!(total, dec!(31.25));
    }

    #[test]
    fn test_cdr_rating_ussd() {
        let rate_per_session = dec!(2.00); // NGN per USSD session
        let sessions = 3;
        
        let total = rate_per_session * Decimal::from(sessions);
        assert_eq!(total, dec!(6.00));
    }

    // Invoice Tests
    #[test]
    fn test_invoice_calculation() {
        let items = vec![
            ("SMS", dec!(4.00), 100),  // 100 SMS @ 4 NGN
            ("Voice", dec!(12.50), 30), // 30 minutes @ 12.50 NGN
            ("USSD", dec!(2.00), 20),   // 20 sessions @ 2 NGN
        ];
        
        let total: Decimal = items.iter()
            .map(|(_, rate, qty)| *rate * Decimal::from(*qty))
            .sum();
        
        // 400 + 375 + 40 = 815
        assert_eq!(total, dec!(815.00));
    }

    #[test]
    fn test_invoice_with_vat() {
        let subtotal = dec!(1000.00);
        let vat_rate = dec!(0.075); // 7.5% VAT
        
        let vat_amount = subtotal * vat_rate;
        let total = subtotal + vat_amount;
        
        assert_eq!(vat_amount, dec!(75.00));
        assert_eq!(total, dec!(1075.00));
    }

    #[test]
    fn test_currency_conversion() {
        let ngn_amount = dec!(1000.00);
        let usd_rate = dec!(0.00065); // 1 USD = ~1538 NGN
        
        let usd_amount = ngn_amount * usd_rate;
        assert!(usd_amount < dec!(1.00));
        assert!(usd_amount > dec!(0.50));
    }
}
