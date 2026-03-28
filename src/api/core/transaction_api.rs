//! Transaction Management API - Core Layer
//!
//! Provides transport layer-independent transaction management capabilities

use crate::api::core::{CoreError, CoreResult, TransactionHandle};
use crate::transaction::{TransactionManager, TransactionOptions};
use std::sync::Arc;

/// Common Transaction API - Core Layer
pub struct TransactionApi {
    txn_manager: Arc<TransactionManager>,
}

impl TransactionApi {
    /// Creating a New Transaction API Instance
    pub fn new(txn_manager: Arc<TransactionManager>) -> Self {
        Self { txn_manager }
    }

    /// Commencement of business
    ///
    /// # Parameters
    /// - `options`: transaction options
    ///
    /// # Back
    /// transaction handle
    pub fn begin(&self, options: TransactionOptions) -> CoreResult<TransactionHandle> {
        let txn_id = self
            .txn_manager
            .begin_transaction(options)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))?;
        Ok(TransactionHandle(txn_id))
    }

    /// Submission of transactions
    ///
    /// # Parameters
    /// - `handle`: transaction handle
    pub fn commit(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .commit_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// Rolling back (aborting) transactions
    ///
    /// # Parameters
    /// `handle`: Transaction handler
    pub fn rollback(&self, handle: TransactionHandle) -> CoreResult<()> {
        self.txn_manager
            .abort_transaction(handle.0)
            .map_err(|e| CoreError::TransactionFailed(e.to_string()))
    }

    /// Getting Transaction Status
    ///
    /// # Parameters
    /// - `handle`: transaction handle
    ///
    /// # Return
    /// Transaction Status String
    pub fn get_status(&self, _handle: TransactionHandle) -> CoreResult<String> {
        // Temporarily return Active, actually need to query the transaction status
        Ok("Active".to_string())
    }

    /// Check if a transaction exists and is active
    ///
    /// # Parameters
    /// - `handle`: transaction handle
    pub fn is_active(&self, handle: TransactionHandle) -> bool {
        self.txn_manager.is_transaction_active(handle.0)
    }

    /// Get the number of active transactions
    pub fn active_count(&self) -> usize {
        // Temporarily return 0, actually need to query
        0
    }
}

impl Clone for TransactionApi {
    fn clone(&self) -> Self {
        Self {
            txn_manager: Arc::clone(&self.txn_manager),
        }
    }
}
