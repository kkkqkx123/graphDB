//! TwoPhaseCoordinator 测试
//!
//! 测试两阶段提交协调器的功能，包括开始2PC事务、记录投票、状态转换等

use std::time::Duration;

use crate::transaction::two_phase::{
    ParticipantState, ParticipantVote, ResourceManager, TwoPhaseCoordinator, TwoPhaseId,
    TwoPhaseState, TwoPhaseTransaction,
};
use crate::transaction::types::{TransactionError, TransactionId};

/// 模拟资源管理器
struct _MockResourceManager {
    id: String,
    should_fail_prepare: bool,
    should_fail_commit: bool,
    should_fail_abort: bool,
}

impl _MockResourceManager {
    fn _new(id: &str) -> Self {
        Self {
            id: id.to_string(),
            should_fail_prepare: false,
            should_fail_commit: false,
            should_fail_abort: false,
        }
    }

    fn _fail_prepare(mut self) -> Self {
        self.should_fail_prepare = true;
        self
    }

    fn _fail_commit(mut self) -> Self {
        self.should_fail_commit = true;
        self
    }

    fn _fail_abort(mut self) -> Self {
        self.should_fail_abort = true;
        self
    }
}

impl ResourceManager for _MockResourceManager {
    fn id(&self) -> &str {
        &self.id
    }

    fn prepare(&self, _txn_id: TransactionId) -> Result<(), String> {
        if self.should_fail_prepare {
            Err(format!("{} prepare failed", self.id))
        } else {
            Ok(())
        }
    }

    fn commit(&self, _txn_id: TransactionId) -> Result<(), String> {
        if self.should_fail_commit {
            Err(format!("{} commit failed", self.id))
        } else {
            Ok(())
        }
    }

    fn abort(&self, _txn_id: TransactionId) -> Result<(), String> {
        if self.should_fail_abort {
            Err(format!("{} abort failed", self.id))
        } else {
            Ok(())
        }
    }
}

#[test]
fn test_two_phase_id_creation() {
    let id = TwoPhaseId::new(1);
    assert_eq!(id.value(), 1);
}

#[test]
fn test_two_phase_id_display() {
    let id = TwoPhaseId::new(42);
    assert_eq!(format!("{}", id), "42");
}

#[test]
fn test_two_phase_id_default() {
    let id = TwoPhaseId::default();
    assert_eq!(id.value(), 0);
}

#[test]
fn test_two_phase_state_can_commit() {
    assert!(!TwoPhaseState::Preparing.can_commit());
    assert!(TwoPhaseState::AllPrepared.can_commit());
    assert!(!TwoPhaseState::VoteAbort.can_commit());
    assert!(!TwoPhaseState::Committing.can_commit());
    assert!(!TwoPhaseState::Committed.can_commit());
    assert!(!TwoPhaseState::Aborting.can_commit());
    assert!(!TwoPhaseState::Aborted.can_commit());
    assert!(!TwoPhaseState::Timeout.can_commit());
}

#[test]
fn test_two_phase_state_can_abort() {
    assert!(TwoPhaseState::Preparing.can_abort());
    assert!(TwoPhaseState::AllPrepared.can_abort());
    assert!(TwoPhaseState::VoteAbort.can_abort());
    assert!(!TwoPhaseState::Committing.can_abort());
    assert!(!TwoPhaseState::Committed.can_abort());
    assert!(!TwoPhaseState::Aborting.can_abort());
    assert!(!TwoPhaseState::Aborted.can_abort());
    assert!(!TwoPhaseState::Timeout.can_abort());
}

#[test]
fn test_two_phase_state_is_terminal() {
    assert!(!TwoPhaseState::Preparing.is_terminal());
    assert!(!TwoPhaseState::AllPrepared.is_terminal());
    assert!(!TwoPhaseState::VoteAbort.is_terminal());
    assert!(!TwoPhaseState::Committing.is_terminal());
    assert!(TwoPhaseState::Committed.is_terminal());
    assert!(!TwoPhaseState::Aborting.is_terminal());
    assert!(TwoPhaseState::Aborted.is_terminal());
    assert!(TwoPhaseState::Timeout.is_terminal());
}

#[test]
fn test_two_phase_state_display() {
    assert_eq!(format!("{}", TwoPhaseState::Preparing), "Preparing");
    assert_eq!(format!("{}", TwoPhaseState::AllPrepared), "AllPrepared");
    assert_eq!(format!("{}", TwoPhaseState::VoteAbort), "VoteAbort");
    assert_eq!(format!("{}", TwoPhaseState::Committing), "Committing");
    assert_eq!(format!("{}", TwoPhaseState::Committed), "Committed");
    assert_eq!(format!("{}", TwoPhaseState::Aborting), "Aborting");
    assert_eq!(format!("{}", TwoPhaseState::Aborted), "Aborted");
    assert_eq!(format!("{}", TwoPhaseState::Timeout), "Timeout");
}

#[test]
fn test_participant_state_creation() {
    let state = ParticipantState::new("rm1".to_string());
    assert_eq!(state.id, "rm1");
    assert!(state.vote.is_none());
    assert!(state.vote_time.is_none());
    assert!(!state.committed);
    assert!(!state.aborted);
}

#[test]
fn test_participant_state_record_vote() {
    let mut state = ParticipantState::new("rm1".to_string());
    state.record_vote(ParticipantVote::Ready);

    assert_eq!(state.vote, Some(ParticipantVote::Ready));
    assert!(state.vote_time.is_some());
}

#[test]
fn test_participant_state_mark_committed() {
    let mut state = ParticipantState::new("rm1".to_string());
    state.mark_committed();

    assert!(state.committed);
    assert!(!state.aborted);
}

#[test]
fn test_participant_state_mark_aborted() {
    let mut state = ParticipantState::new("rm1".to_string());
    state.mark_aborted();

    assert!(!state.committed);
    assert!(state.aborted);
}

#[test]
fn test_two_phase_transaction_creation() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];
    let timeout = Duration::from_secs(30);

    let txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    assert_eq!(txn.id, id);
    assert_eq!(txn.txn_id, txn_id);
    assert_eq!(txn.participants.len(), 2);
    assert_eq!(txn.state, TwoPhaseState::Preparing);
    assert!(!txn.is_expired());
}

#[test]
fn test_two_phase_transaction_expiration() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];
    let timeout = Duration::from_millis(50);

    let txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    // 初始不应该超时
    assert!(!txn.is_expired());

    // 等待超时
    std::thread::sleep(Duration::from_millis(100));

    // 现在应该超时
    assert!(txn.is_expired());
}

#[test]
fn test_two_phase_transaction_record_vote() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];
    let timeout = Duration::from_secs(30);

    let mut txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    // 记录第一个投票
    txn.record_vote("rm1", ParticipantVote::Ready);

    assert_eq!(txn.state, TwoPhaseState::Preparing);
    assert_eq!(txn.pending_count(), 1);
    assert_eq!(txn.ready_count(), 1);
    assert_eq!(txn.abort_count(), 0);

    // 记录第二个投票
    txn.record_vote("rm2", ParticipantVote::Ready);

    // 所有参与者都投票准备，状态应该变为 AllPrepared
    assert_eq!(txn.state, TwoPhaseState::AllPrepared);
    assert_eq!(txn.pending_count(), 0);
    assert_eq!(txn.ready_count(), 2);
    assert_eq!(txn.abort_count(), 0);
}

#[test]
fn test_two_phase_transaction_vote_abort() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string()];
    let timeout = Duration::from_secs(30);

    let mut txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    // 第一个参与者投票准备
    txn.record_vote("rm1", ParticipantVote::Ready);

    // 第二个参与者投票中止
    txn.record_vote("rm2", ParticipantVote::Abort);

    // 状态应该变为 VoteAbort
    assert_eq!(txn.state, TwoPhaseState::VoteAbort);
    assert_eq!(txn.pending_count(), 0);
    assert_eq!(txn.ready_count(), 1);
    assert_eq!(txn.abort_count(), 1);
}

#[test]
fn test_two_phase_transaction_ready_count() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string(), "rm3".to_string()];
    let timeout = Duration::from_secs(30);

    let mut txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    assert_eq!(txn.ready_count(), 0);

    txn.record_vote("rm1", ParticipantVote::Ready);
    assert_eq!(txn.ready_count(), 1);

    txn.record_vote("rm2", ParticipantVote::Ready);
    assert_eq!(txn.ready_count(), 2);

    txn.record_vote("rm3", ParticipantVote::Ready);
    assert_eq!(txn.ready_count(), 3);
}

#[test]
fn test_two_phase_transaction_abort_count() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string(), "rm3".to_string()];
    let timeout = Duration::from_secs(30);

    let mut txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    assert_eq!(txn.abort_count(), 0);

    txn.record_vote("rm1", ParticipantVote::Abort);
    assert_eq!(txn.abort_count(), 1);

    txn.record_vote("rm2", ParticipantVote::Abort);
    assert_eq!(txn.abort_count(), 2);

    txn.record_vote("rm3", ParticipantVote::Ready);
    assert_eq!(txn.abort_count(), 2);
}

#[test]
fn test_two_phase_transaction_pending_count() {
    let id = TwoPhaseId::new(1);
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string(), "rm2".to_string(), "rm3".to_string()];
    let timeout = Duration::from_secs(30);

    let mut txn = TwoPhaseTransaction::new(id, txn_id, participant_ids, timeout);

    assert_eq!(txn.pending_count(), 3);

    txn.record_vote("rm1", ParticipantVote::Ready);
    assert_eq!(txn.pending_count(), 2);

    txn.record_vote("rm2", ParticipantVote::Ready);
    assert_eq!(txn.pending_count(), 1);

    txn.record_vote("rm3", ParticipantVote::Ready);
    assert_eq!(txn.pending_count(), 0);
}

#[test]
fn test_two_phase_coordinator_creation() {
    let coordinator = TwoPhaseCoordinator::new(Duration::from_secs(30));

    // 验证协调器创建成功
    assert!(coordinator.list_active_transactions().is_empty());
}

#[test]
fn test_two_phase_coordinator_default() {
    let coordinator = TwoPhaseCoordinator::default();

    // 验证协调器创建成功
    assert!(coordinator.list_active_transactions().is_empty());
}

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
fn test_begin_two_phase_with_timeout() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];
    let timeout = Duration::from_secs(60);

    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, Some(timeout))
        .expect("开始2PC失败");

    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert_eq!(txn.timeout, timeout);
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

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::AllPrepared));
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

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::VoteAbort));
}

#[test]
fn test_record_vote_timeout() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];
    let timeout = Duration::from_millis(50);

    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, Some(timeout))
        .expect("开始2PC失败");

    // 等待事务超时
    std::thread::sleep(Duration::from_millis(100));

    // 记录投票应该失败，因为事务已超时
    let result = coordinator.record_vote(two_phase_id, "rm1", ParticipantVote::Ready);
    assert!(matches!(
        result,
        Err(TransactionError::TransactionTimeout)
    ));

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::Timeout));
}

#[test]
fn test_record_vote_nonexistent_transaction() {
    let coordinator = TwoPhaseCoordinator::default();
    let two_phase_id = TwoPhaseId::new(999);

    let result = coordinator.record_vote(two_phase_id, "rm1", ParticipantVote::Ready);
    assert!(matches!(
        result,
        Err(TransactionError::TwoPhaseNotFound(_))
    ));
}

#[test]
fn test_mark_committing() {
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

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::Committing));
}

#[test]
fn test_mark_committing_invalid_state() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];

    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, None)
        .expect("开始2PC失败");

    // 还没有所有参与者投票，不能标记为提交中
    let result = coordinator.mark_committing(two_phase_id);
    assert!(result.is_err());
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

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::Committed));

    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert!(txn.participants.values().all(|p| p.committed));
}

#[test]
fn test_mark_aborting() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];

    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, None)
        .expect("开始2PC失败");

    coordinator
        .mark_aborting(two_phase_id)
        .expect("标记中止中失败");

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::Aborting));
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
        .mark_aborting(two_phase_id)
        .expect("标记中止中失败");

    coordinator
        .mark_aborted(two_phase_id)
        .expect("标记已中止失败");

    let state = coordinator.get_state(two_phase_id);
    assert_eq!(state, Some(TwoPhaseState::Aborted));

    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert!(txn.participants.values().all(|p| p.aborted));
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
fn test_get_two_phase_id_nonexistent() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 999;

    let found = coordinator.get_two_phase_id(txn_id);
    assert_eq!(found, None);
}

#[test]
fn test_get_transaction() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn_id: TransactionId = 1;
    let participant_ids = vec!["rm1".to_string()];

    let two_phase_id = coordinator
        .begin_two_phase(txn_id, participant_ids, None)
        .expect("开始2PC失败");

    let txn = coordinator
        .get_transaction(two_phase_id)
        .expect("获取事务失败");
    assert_eq!(txn.txn_id, txn_id);
}

#[test]
fn test_get_transaction_nonexistent() {
    let coordinator = TwoPhaseCoordinator::default();
    let two_phase_id = TwoPhaseId::new(999);

    let txn = coordinator.get_transaction(two_phase_id);
    assert!(txn.is_none());
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

    let found = coordinator.get_two_phase_id(txn_id);
    assert_eq!(found, None);
}

#[test]
fn test_list_active_transactions() {
    let coordinator = TwoPhaseCoordinator::default();

    let txn1_id: TransactionId = 1;
    let txn2_id: TransactionId = 2;

    let two_phase_id1 = coordinator
        .begin_two_phase(txn1_id, vec!["rm1".to_string()], None)
        .expect("开始2PC失败");

    let _two_phase_id2 = coordinator
        .begin_two_phase(txn2_id, vec!["rm2".to_string()], None)
        .expect("开始2PC失败");

    let active_txns = coordinator.list_active_transactions();
    assert_eq!(active_txns.len(), 2);

    // 标记第一个事务为已提交
    coordinator
        .record_vote(two_phase_id1, "rm1", ParticipantVote::Ready)
        .expect("记录投票失败");
    coordinator
        .mark_committing(two_phase_id1)
        .expect("标记提交中失败");
    coordinator
        .mark_committed(two_phase_id1)
        .expect("标记已提交失败");

    // 现在应该只有一个活跃事务
    let active_txns = coordinator.list_active_transactions();
    assert_eq!(active_txns.len(), 1);
}

#[test]
fn test_check_timeouts() {
    let coordinator = TwoPhaseCoordinator::default();
    let txn1_id: TransactionId = 1;
    let txn2_id: TransactionId = 2;

    // 创建一个短超时的事务
    let two_phase_id1 = coordinator
        .begin_two_phase(
            txn1_id,
            vec!["rm1".to_string()],
            Some(Duration::from_millis(50)),
        )
        .expect("开始2PC失败");

    // 创建一个长超时的事务
    let two_phase_id2 = coordinator
        .begin_two_phase(
            txn2_id,
            vec!["rm2".to_string()],
            Some(Duration::from_secs(60)),
        )
        .expect("开始2PC失败");

    // 等待第一个事务超时
    std::thread::sleep(Duration::from_millis(100));

    // 检查超时
    coordinator.check_timeouts();

    // 第一个事务应该超时
    let state1 = coordinator.get_state(two_phase_id1);
    assert_eq!(state1, Some(TwoPhaseState::Timeout));

    // 第二个事务应该仍然活跃
    let state2 = coordinator.get_state(two_phase_id2);
    assert_eq!(state2, Some(TwoPhaseState::Preparing));
}

#[test]
fn test_multiple_two_phase_transactions() {
    let coordinator = TwoPhaseCoordinator::default();

    let txn1_id: TransactionId = 1;
    let txn2_id: TransactionId = 2;
    let txn3_id: TransactionId = 3;

    let two_phase_id1 = coordinator
        .begin_two_phase(txn1_id, vec!["rm1".to_string()], None)
        .expect("开始2PC失败");

    let two_phase_id2 = coordinator
        .begin_two_phase(txn2_id, vec!["rm2".to_string()], None)
        .expect("开始2PC失败");

    let two_phase_id3 = coordinator
        .begin_two_phase(txn3_id, vec!["rm3".to_string()], None)
        .expect("开始2PC失败");

    // 验证每个事务都有正确的ID
    assert_eq!(two_phase_id1.value(), 1);
    assert_eq!(two_phase_id2.value(), 2);
    assert_eq!(two_phase_id3.value(), 3);

    // 验证事务ID映射
    assert_eq!(coordinator.get_two_phase_id(txn1_id), Some(two_phase_id1));
    assert_eq!(coordinator.get_two_phase_id(txn2_id), Some(two_phase_id2));
    assert_eq!(coordinator.get_two_phase_id(txn3_id), Some(two_phase_id3));
}
