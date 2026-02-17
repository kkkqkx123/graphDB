//! 两阶段提交（2PC）协调器模块
//!
//! 提供分布式事务的两阶段提交支持，确保跨多个资源的事务一致性

use std::collections::HashMap;
use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use parking_lot::RwLock;

use crate::transaction::{TransactionError, TransactionId, TransactionState};

/// 2PC事务ID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TwoPhaseId(u64);

impl TwoPhaseId {
    /// 创建新的2PC事务ID
    pub fn new(id: u64) -> Self {
        Self(id)
    }

    /// 获取原始ID值
    pub fn value(&self) -> u64 {
        self.0
    }
}

impl fmt::Display for TwoPhaseId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Default for TwoPhaseId {
    fn default() -> Self {
        Self(0)
    }
}

/// 2PC事务状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TwoPhaseState {
    /// 准备阶段1：正在收集参与者投票
    Preparing,
    /// 所有参与者已投票准备
    AllPrepared,
    /// 至少一个参与者投票中止
    VoteAbort,
    /// 提交阶段2：正在提交
    Committing,
    /// 已提交
    Committed,
    /// 正在中止
    Aborting,
    /// 已中止
    Aborted,
    /// 超时
    Timeout,
}

impl TwoPhaseState {
    /// 检查是否可以提交
    pub fn can_commit(&self) -> bool {
        matches!(self, TwoPhaseState::AllPrepared)
    }

    /// 检查是否可以中止
    pub fn can_abort(&self) -> bool {
        matches!(
            self,
            TwoPhaseState::Preparing
                | TwoPhaseState::AllPrepared
                | TwoPhaseState::VoteAbort
        )
    }

    /// 检查是否已结束
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            TwoPhaseState::Committed | TwoPhaseState::Aborted | TwoPhaseState::Timeout
        )
    }
}

impl fmt::Display for TwoPhaseState {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TwoPhaseState::Preparing => write!(f, "Preparing"),
            TwoPhaseState::AllPrepared => write!(f, "AllPrepared"),
            TwoPhaseState::VoteAbort => write!(f, "VoteAbort"),
            TwoPhaseState::Committing => write!(f, "Committing"),
            TwoPhaseState::Committed => write!(f, "Committed"),
            TwoPhaseState::Aborting => write!(f, "Aborting"),
            TwoPhaseState::Aborted => write!(f, "Aborted"),
            TwoPhaseState::Timeout => write!(f, "Timeout"),
        }
    }
}

/// 参与者投票
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParticipantVote {
    /// 准备就绪，可以提交
    Ready,
    /// 无法准备，需要中止
    Abort,
    /// 超时未响应
    Timeout,
}

/// 参与者状态
#[derive(Debug, Clone)]
pub struct ParticipantState {
    /// 参与者ID
    pub id: String,
    /// 投票状态
    pub vote: Option<ParticipantVote>,
    /// 投票时间
    pub vote_time: Option<Instant>,
    /// 是否已提交
    pub committed: bool,
    /// 是否已中止
    pub aborted: bool,
}

impl ParticipantState {
    /// 创建新的参与者状态
    pub fn new(id: String) -> Self {
        Self {
            id,
            vote: None,
            vote_time: None,
            committed: false,
            aborted: false,
        }
    }

    /// 记录投票
    pub fn record_vote(&mut self, vote: ParticipantVote) {
        self.vote = Some(vote);
        self.vote_time = Some(Instant::now());
    }

    /// 标记为已提交
    pub fn mark_committed(&mut self) {
        self.committed = true;
    }

    /// 标记为已中止
    pub fn mark_aborted(&mut self) {
        self.aborted = true;
    }
}

/// 2PC事务信息
#[derive(Debug, Clone)]
pub struct TwoPhaseTransaction {
    /// 2PC事务ID
    pub id: TwoPhaseId,
    /// 关联的事务ID
    pub txn_id: TransactionId,
    /// 当前状态
    pub state: TwoPhaseState,
    /// 参与者列表
    pub participants: HashMap<String, ParticipantState>,
    /// 创建时间
    pub created_at: Instant,
    /// 超时时间
    pub timeout: Duration,
    /// 准备阶段完成时间
    pub prepared_at: Option<Instant>,
    /// 提交/中止完成时间
    pub completed_at: Option<Instant>,
}

impl TwoPhaseTransaction {
    /// 创建新的2PC事务
    pub fn new(
        id: TwoPhaseId,
        txn_id: TransactionId,
        participant_ids: Vec<String>,
        timeout: Duration,
    ) -> Self {
        let participants = participant_ids
            .into_iter()
            .map(|id| (id.clone(), ParticipantState::new(id)))
            .collect();

        Self {
            id,
            txn_id,
            state: TwoPhaseState::Preparing,
            participants,
            created_at: Instant::now(),
            timeout,
            prepared_at: None,
            completed_at: None,
        }
    }

    /// 检查是否已超时
    pub fn is_expired(&self) -> bool {
        self.created_at.elapsed() > self.timeout
    }

    /// 记录参与者投票
    pub fn record_vote(&mut self, participant_id: &str, vote: ParticipantVote) {
        if let Some(participant) = self.participants.get_mut(participant_id) {
            participant.record_vote(vote);
        }

        // 检查是否所有参与者都已投票
        self.check_all_voted();
    }

    /// 检查是否所有参与者都已投票
    fn check_all_voted(&mut self) {
        let all_voted = self
            .participants
            .values()
            .all(|p| p.vote.is_some());

        if all_voted && matches!(self.state, TwoPhaseState::Preparing) {
            // 检查是否有任何中止投票
            let any_abort = self
                .participants
                .values()
                .any(|p| matches!(p.vote, Some(ParticipantVote::Abort)));

            if any_abort {
                self.state = TwoPhaseState::VoteAbort;
            } else {
                self.state = TwoPhaseState::AllPrepared;
                self.prepared_at = Some(Instant::now());
            }
        }
    }

    /// 获取已准备就绪的参与者数量
    pub fn ready_count(&self) -> usize {
        self.participants
            .values()
            .filter(|p| matches!(p.vote, Some(ParticipantVote::Ready)))
            .count()
    }

    /// 获取已投票中止的参与者数量
    pub fn abort_count(&self) -> usize {
        self.participants
            .values()
            .filter(|p| matches!(p.vote, Some(ParticipantVote::Abort)))
            .count()
    }

    /// 获取尚未投票的参与者数量
    pub fn pending_count(&self) -> usize {
        self.participants
            .values()
            .filter(|p| p.vote.is_none())
            .count()
    }
}

/// 资源管理器trait
///
/// 定义参与2PC的资源的接口
pub trait ResourceManager: Send + Sync {
    /// 获取资源管理器ID
    fn id(&self) -> &str;

    /// 准备阶段1：准备提交
    ///
    /// # Returns
    /// * `Ok(())` - 准备成功，可以提交
    /// * `Err(String)` - 准备失败，需要中止
    fn prepare(&self, txn_id: TransactionId) -> Result<(), String>;

    /// 提交阶段2：提交事务
    ///
    /// # Returns
    /// * `Ok(())` - 提交成功
    /// * `Err(String)` - 提交失败
    fn commit(&self, txn_id: TransactionId) -> Result<(), String>;

    /// 中止事务
    ///
    /// # Returns
    /// * `Ok(())` - 中止成功
    /// * `Err(String)` - 中止失败
    fn abort(&self, txn_id: TransactionId) -> Result<(), String>;
}

/// 2PC协调器
///
/// 管理两阶段提交协议的执行
pub struct TwoPhaseCoordinator {
    /// 2PC事务ID生成器
    id_generator: AtomicU64,
    /// 活跃的2PC事务
    transactions: RwLock<HashMap<TwoPhaseId, Arc<RwLock<TwoPhaseTransaction>>>>,
    /// 事务ID到2PC事务ID的映射
    txn_to_2pc: RwLock<HashMap<TransactionId, TwoPhaseId>>,
    /// 默认超时时间
    default_timeout: Duration,
}

impl TwoPhaseCoordinator {
    /// 创建新的2PC协调器
    pub fn new(default_timeout: Duration) -> Self {
        Self {
            id_generator: AtomicU64::new(1),
            transactions: RwLock::new(HashMap::new()),
            txn_to_2pc: RwLock::new(HashMap::new()),
            default_timeout,
        }
    }

    /// 开始2PC事务
    ///
    /// # Arguments
    /// * `txn_id` - 关联的事务ID
    /// * `participant_ids` - 参与者ID列表
    /// * `timeout` - 超时时间（可选，使用默认值）
    ///
    /// # Returns
    /// * `Ok(TwoPhaseId)` - 2PC事务ID
    pub fn begin_two_phase(
        &self,
        txn_id: TransactionId,
        participant_ids: Vec<String>,
        timeout: Option<Duration>,
    ) -> Result<TwoPhaseId, TransactionError> {
        let id = TwoPhaseId::new(self.id_generator.fetch_add(1, Ordering::SeqCst));
        let timeout = timeout.unwrap_or(self.default_timeout);

        let txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

        self.transactions.write().insert(id, Arc::new(RwLock::new(txn)));
        self.txn_to_2pc.write().insert(txn_id, id);

        Ok(id)
    }

    /// 记录参与者投票
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    /// * `participant_id` - 参与者ID
    /// * `vote` - 投票
    pub fn record_vote(
        &self,
        two_phase_id: TwoPhaseId,
        participant_id: &str,
        vote: ParticipantVote,
    ) -> Result<(), TransactionError> {
        let txn = self
            .transactions
            .read()
            .get(&two_phase_id)
            .cloned()
            .ok_or(TransactionError::TwoPhaseNotFound(two_phase_id))?;

        let mut txn_write = txn.write();

        // 检查是否已超时
        if txn_write.is_expired() {
            txn_write.state = TwoPhaseState::Timeout;
            return Err(TransactionError::TransactionTimeout);
        }

        txn_write.record_vote(participant_id, vote);

        Ok(())
    }

    /// 获取2PC事务状态
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn get_state(&self, two_phase_id: TwoPhaseId) -> Option<TwoPhaseState> {
        self.transactions
            .read()
            .get(&two_phase_id)
            .map(|txn| txn.read().state)
    }

    /// 检查是否可以提交
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn can_commit(&self, two_phase_id: TwoPhaseId) -> bool {
        self.get_state(two_phase_id)
            .map(|state| state.can_commit())
            .unwrap_or(false)
    }

    /// 检查是否可以中止
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn can_abort(&self, two_phase_id: TwoPhaseId) -> bool {
        self.get_state(two_phase_id)
            .map(|state| state.can_abort())
            .unwrap_or(false)
    }

    /// 标记为提交中
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn mark_committing(&self, two_phase_id: TwoPhaseId) -> Result<(), TransactionError> {
        let txn = self
            .transactions
            .read()
            .get(&two_phase_id)
            .cloned()
            .ok_or(TransactionError::TwoPhaseNotFound(two_phase_id))?;

        let mut txn_write = txn.write();

        if !txn_write.state.can_commit() {
            return Err(TransactionError::InvalidStateForCommit(TransactionState::Prepared));
        }

        txn_write.state = TwoPhaseState::Committing;
        Ok(())
    }

    /// 标记为已提交
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn mark_committed(&self, two_phase_id: TwoPhaseId) -> Result<(), TransactionError> {
        let txn = self
            .transactions
            .read()
            .get(&two_phase_id)
            .cloned()
            .ok_or(TransactionError::TwoPhaseNotFound(two_phase_id))?;

        let mut txn_write = txn.write();
        txn_write.state = TwoPhaseState::Committed;
        txn_write.completed_at = Some(Instant::now());

        // 标记所有参与者为已提交
        for participant in txn_write.participants.values_mut() {
            participant.mark_committed();
        }

        Ok(())
    }

    /// 标记为中止中
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn mark_aborting(&self, two_phase_id: TwoPhaseId) -> Result<(), TransactionError> {
        let txn = self
            .transactions
            .read()
            .get(&two_phase_id)
            .cloned()
            .ok_or(TransactionError::TwoPhaseNotFound(two_phase_id))?;

        let mut txn_write = txn.write();

        if !txn_write.state.can_abort() {
            return Err(TransactionError::InvalidStateForAbort(TransactionState::Prepared));
        }

        txn_write.state = TwoPhaseState::Aborting;
        Ok(())
    }

    /// 标记为已中止
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn mark_aborted(&self, two_phase_id: TwoPhaseId) -> Result<(), TransactionError> {
        let txn = self
            .transactions
            .read()
            .get(&two_phase_id)
            .cloned()
            .ok_or(TransactionError::TwoPhaseNotFound(two_phase_id))?;

        let mut txn_write = txn.write();
        txn_write.state = TwoPhaseState::Aborted;
        txn_write.completed_at = Some(Instant::now());

        // 标记所有参与者为已中止
        for participant in txn_write.participants.values_mut() {
            participant.mark_aborted();
        }

        Ok(())
    }

    /// 通过事务ID获取2PC事务ID
    ///
    /// # Arguments
    /// * `txn_id` - 事务ID
    pub fn get_two_phase_id(&self, txn_id: TransactionId) -> Option<TwoPhaseId> {
        self.txn_to_2pc.read().get(&txn_id).copied()
    }

    /// 获取2PC事务信息
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn get_transaction(&self, two_phase_id: TwoPhaseId) -> Option<TwoPhaseTransaction> {
        self.transactions
            .read()
            .get(&two_phase_id)
            .map(|txn| txn.read().clone())
    }

    /// 清理已完成的2PC事务
    ///
    /// # Arguments
    /// * `two_phase_id` - 2PC事务ID
    pub fn cleanup_transaction(&self, two_phase_id: TwoPhaseId) {
        if let Some(txn) = self.transactions.write().remove(&two_phase_id) {
            let txn_read = txn.read();
            let txn_id = txn_read.txn_id;
            drop(txn_read);

            self.txn_to_2pc.write().remove(&txn_id);
        }
    }

    /// 获取所有活跃的2PC事务
    pub fn list_active_transactions(&self) -> Vec<TwoPhaseTransaction> {
        self.transactions
            .read()
            .values()
            .filter_map(|txn| {
                let txn_read = txn.read();
                if !txn_read.state.is_terminal() {
                    Some(txn_read.clone())
                } else {
                    None
                }
            })
            .collect()
    }

    /// 检查并处理超时事务
    pub fn check_timeouts(&self) {
        let expired: Vec<TwoPhaseId> = self
            .transactions
            .read()
            .iter()
            .filter_map(|(id, txn)| {
                let txn_read = txn.read();
                if txn_read.is_expired() && !txn_read.state.is_terminal() {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect();

        for id in expired {
            if let Some(txn) = self.transactions.read().get(&id) {
                txn.write().state = TwoPhaseState::Timeout;
            }
        }
    }
}

impl Default for TwoPhaseCoordinator {
    fn default() -> Self {
        Self::new(Duration::from_secs(30))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_begin_two_phase() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        assert_eq!(two_phase_id.value(), 1);

        let txn = coordinator
            .get_transaction(two_phase_id)
            .expect("获取事务失败");
        assert_eq!(txn.txn_id, txn_id);
        assert_eq!(txn.participants.len(), 2);
    }

    #[test]
    fn test_record_vote_all_ready() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        // 记录第一个参与者的投票
        coordinator
            .record_vote(two_phase_id, "rm1", ParticipantVote::Ready)
            .expect("记录投票失败");

        assert!(!coordinator.can_commit(two_phase_id));

        // 记录第二个参与者的投票
        coordinator
            .record_vote(two_phase_id, "rm2", ParticipantVote::Ready)
            .expect("记录投票失败");

        assert!(coordinator.can_commit(two_phase_id));
    }

    #[test]
    fn test_record_vote_abort() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        // 第一个参与者投票准备
        coordinator
            .record_vote(two_phase_id, "rm1", ParticipantVote::Ready)
            .expect("记录投票失败");

        // 第二个参与者投票中止
        coordinator
            .record_vote(two_phase_id, "rm2", ParticipantVote::Abort)
            .expect("记录投票失败");

        assert!(!coordinator.can_commit(two_phase_id));
        assert!(coordinator.can_abort(two_phase_id));
    }

    #[test]
    fn test_mark_committed() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        coordinator
            .record_vote(two_phase_id, "rm1", ParticipantVote::Ready)
            .expect("记录投票失败");

        coordinator
            .mark_committing(two_phase_id)
            .expect("标记提交中失败");

        coordinator
            .mark_committed(two_phase_id)
            .expect("标记已提交失败");

        let txn = coordinator
            .get_transaction(two_phase_id)
            .expect("获取事务失败");
        assert!(matches!(txn.state, TwoPhaseState::Committed));
    }

    #[test]
    fn test_mark_aborted() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        coordinator
            .record_vote(two_phase_id, "rm1", ParticipantVote::Abort)
            .expect("记录投票失败");

        coordinator
            .mark_aborting(two_phase_id)
            .expect("标记中止中失败");

        coordinator
            .mark_aborted(two_phase_id)
            .expect("标记已中止失败");

        let txn = coordinator
            .get_transaction(two_phase_id)
            .expect("获取事务失败");
        assert!(matches!(txn.state, TwoPhaseState::Aborted));
    }

    #[test]
    fn test_get_two_phase_id() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        let found = coordinator.get_two_phase_id(txn_id);
        assert_eq!(found, Some(two_phase_id));
    }

    #[test]
    fn test_cleanup_transaction() {
        let coordinator = TwoPhaseCoordinator::default();
        let txn_id: TransactionId = 1;
        let participant_ids = vec!["rm1".to_string()];

        let two_phase_id = coordinator
            .begin_two_phase(txn_id, participant_ids, None)
            .expect("开始2PC失败");

        coordinator.cleanup_transaction(two_phase_id);

        let txn = coordinator.get_transaction(two_phase_id);
        assert!(txn.is_none());
    }
}
