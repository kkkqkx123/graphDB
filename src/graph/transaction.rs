use crate::core::{Edge, Value, Vertex};
use crate::storage::TransactionId;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum TransactionError {
    #[error("Transaction {0} not found")]
    TransactionNotFound(TransactionId),
    #[error("Transaction already committed: {0}")]
    AlreadyCommitted(TransactionId),
    #[error("Transaction already rolled back: {0}")]
    AlreadyRolledBack(TransactionId),
    #[error("Storage error: {0}")]
    StorageError(String),
}

pub enum Operation {
    InsertNode(Vertex),
    UpdateNode(Vertex),
    DeleteNode(Value),
    InsertEdge(Edge),
    DeleteEdge(Value, Value, String), // Changed to match the new Edge structure
}

pub struct Transaction {
    id: TransactionId,
    operations: Vec<Operation>,
    committed: bool,
    rolled_back: bool,
}

impl Transaction {
    pub fn new(id: TransactionId) -> Self {
        Self {
            id,
            operations: Vec::new(),
            committed: false,
            rolled_back: false,
        }
    }

    pub fn add_operation(&mut self, operation: Operation) -> Result<(), TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted(self.id));
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack(self.id));
        }

        self.operations.push(operation);
        Ok(())
    }

    pub fn commit(&mut self) -> Result<(), TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted(self.id));
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack(self.id));
        }

        self.committed = true;
        Ok(())
    }

    pub fn rollback(&mut self) -> Result<(), TransactionError> {
        if self.committed {
            return Err(TransactionError::AlreadyCommitted(self.id));
        }
        if self.rolled_back {
            return Err(TransactionError::AlreadyRolledBack(self.id));
        }

        self.rolled_back = true;
        Ok(())
    }

    pub fn is_active(&self) -> bool {
        !self.committed && !self.rolled_back
    }
}

pub struct TransactionManager {
    current_tx_id: AtomicU64,
    active_transactions: HashMap<TransactionId, Transaction>,
}

impl TransactionManager {
    pub fn new() -> Self {
        Self {
            current_tx_id: AtomicU64::new(1),
            active_transactions: HashMap::new(),
        }
    }

    pub fn begin_transaction(&mut self) -> TransactionId {
        let tx_id = self.current_tx_id.fetch_add(1, Ordering::SeqCst);
        let transaction = Transaction::new(tx_id);
        self.active_transactions.insert(tx_id, transaction);
        tx_id
    }

    pub fn get_transaction(
        &mut self,
        tx_id: TransactionId,
    ) -> Result<&mut Transaction, TransactionError> {
        self.active_transactions
            .get_mut(&tx_id)
            .ok_or_else(|| TransactionError::TransactionNotFound(tx_id))
    }

    pub fn commit_transaction(&mut self, tx_id: TransactionId) -> Result<(), TransactionError> {
        let transaction = self.get_transaction(tx_id)?;
        transaction.commit()?;

        // In a real implementation, we would apply the operations to the storage
        // For now, we'll just remove the transaction
        self.active_transactions.remove(&tx_id);

        Ok(())
    }

    pub fn rollback_transaction(&mut self, tx_id: TransactionId) -> Result<(), TransactionError> {
        let transaction = self.get_transaction(tx_id)?;
        transaction.rollback()?;

        // In a real implementation, we would undo the operations on the storage
        // For now, we'll just remove the transaction
        self.active_transactions.remove(&tx_id);

        Ok(())
    }
}
