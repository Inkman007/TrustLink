#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::{Address as _, Ledger}, Address, Env, String};

fn create_test_contract(env: &Env) -> (Address, TrustLinkContractClient) {
    let contract_id = env.register_contract(None, TrustLinkContract);
    let client = TrustLinkContractClient::new(env, &contract_id);
    (contract_id, client)
}

#[test]
fn test_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    
    let stored_admin = client.get_admin();
    assert_eq!(stored_admin, admin);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_double_initialization() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.initialize(&admin); // Should panic
}

#[test]
fn test_register_and_check_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    assert!(client.is_issuer(&issuer));
}

#[test]
fn test_remove_issuer() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    assert!(client.is_issuer(&issuer));
    
    client.remove_issuer(&admin, &issuer);
    assert!(!client.is_issuer(&issuer));
}

#[test]
fn test_create_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    
    let attestation = client.get_attestation(&attestation_id);
    assert_eq!(attestation.issuer, issuer);
    assert_eq!(attestation.subject, subject);
    assert_eq!(attestation.claim_type, claim_type);
    assert!(!attestation.revoked);
}

#[test]
fn test_has_valid_claim() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    let other_claim = String::from_str(&env, "ACCREDITED");
    assert!(!client.has_valid_claim(&subject, &other_claim));
}

#[test]
fn test_revoke_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    client.revoke_attestation(&issuer, &attestation_id);
    
    assert!(!client.has_valid_claim(&subject, &claim_type));
    
    let attestation = client.get_attestation(&attestation_id);
    assert!(attestation.revoked);
}

#[test]
fn test_expired_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    let current_time = env.ledger().timestamp();
    let expiration = Some(current_time + 100);
    
    let attestation_id = client.create_attestation(&issuer, &subject, &claim_type, &expiration, &None);
    
    // Should be valid initially
    assert!(client.has_valid_claim(&subject, &claim_type));
    
    // Fast forward time past expiration
    env.ledger().with_mut(|li| {
        li.timestamp = current_time + 200;
    });
    
    // Should now be invalid
    assert!(!client.has_valid_claim(&subject, &claim_type));
    
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Expired);
}

#[test]
#[should_panic(expected = "Error(Contract, #6)")]
fn test_duplicate_attestation() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    let claim_type = String::from_str(&env, "KYC_PASSED");
    
    // Mock the timestamp to be consistent
    env.ledger().with_mut(|li| {
        li.timestamp = 1000;
    });
    
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None);
    client.create_attestation(&issuer, &subject, &claim_type, &None, &None); // Should panic
}

#[test]
fn test_pagination() {
    let env = Env::default();
    env.mock_all_auths();
    
    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);
    
    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);
    
    // Create multiple attestations
    let claims = ["CLAIM_0", "CLAIM_1", "CLAIM_2", "CLAIM_3", "CLAIM_4"];
    for claim_str in claims.iter() {
        let claim = String::from_str(&env, claim_str);
        client.create_attestation(&issuer, &subject, &claim, &None, &None);
    }
    
    let page1 = client.get_subject_attestations(&subject, &0, &2);
    assert_eq!(page1.len(), 2);
    
    let page2 = client.get_subject_attestations(&subject, &2, &2);
    assert_eq!(page2.len(), 2);
    
    let page3 = client.get_subject_attestations(&subject, &4, &2);
    assert_eq!(page3.len(), 1);
}

// ── Task 5.1 ──────────────────────────────────────────────────────────────────
// Requirements: 3.2, 4.1
#[test]
fn test_create_attestation_with_valid_from() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time = env.ledger().timestamp();
    let future_time = current_time + 1000;
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &Some(future_time));

    let attestation = client.get_attestation(&attestation_id);
    assert_eq!(attestation.valid_from, Some(future_time));

    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Pending);
}

// ── Task 5.2 ──────────────────────────────────────────────────────────────────
// Requirements: 2.3, 2.4, 4.1, 4.2
#[test]
fn test_get_status_pending_transitions_to_valid() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let future_time = current_time + 500;
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &Some(future_time));

    // Before valid_from: status must be Pending
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Pending);

    // Advance ledger time past valid_from
    env.ledger().with_mut(|l| l.timestamp = future_time + 1);

    // After valid_from: status must be Valid
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Valid);
}

// ── Task 5.3 ──────────────────────────────────────────────────────────────────
// Requirements: 5.1, 5.3
#[test]
fn test_has_valid_claim_pending_then_valid() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let future_time = current_time + 500;
    let claim_type = String::from_str(&env, "ACCREDITED_INVESTOR");

    client.create_attestation(&issuer, &subject, &claim_type, &None, &Some(future_time));

    // Before valid_from: has_valid_claim must be false
    assert!(!client.has_valid_claim(&subject, &claim_type));

    // Advance ledger time past valid_from
    env.ledger().with_mut(|l| l.timestamp = future_time + 1);

    // After valid_from: has_valid_claim must be true
    assert!(client.has_valid_claim(&subject, &claim_type));
}

// ── Task 5.4 ──────────────────────────────────────────────────────────────────
// Requirements: 6.1, 6.2, 6.3
#[test]
fn test_create_attestation_valid_from_none_unchanged() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let claim_type = String::from_str(&env, "KYC_PASSED");

    // Create with valid_from = None — backward-compatible path
    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &None);

    let attestation = client.get_attestation(&attestation_id);
    assert_eq!(attestation.valid_from, None);

    // Status must be Valid (not Pending)
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Valid);

    // has_valid_claim must return true
    assert!(client.has_valid_claim(&subject, &claim_type));
}

// ── Task 5.5 ──────────────────────────────────────────────────────────────────
// Requirements: 3.4
#[test]
fn test_create_attestation_valid_from_past_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 2_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let past_time = current_time - 1;
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let result = client.try_create_attestation(
        &issuer,
        &subject,
        &claim_type,
        &None,
        &Some(past_time),
    );
    assert_eq!(
        result,
        Err(Ok(types::Error::InvalidValidFrom))
    );
}

#[test]
fn test_create_attestation_valid_from_equal_current_time_rejected() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 2_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let claim_type = String::from_str(&env, "KYC_PASSED");

    // valid_from == current_time must also be rejected
    let result = client.try_create_attestation(
        &issuer,
        &subject,
        &claim_type,
        &None,
        &Some(current_time),
    );
    assert_eq!(
        result,
        Err(Ok(types::Error::InvalidValidFrom))
    );
}

// ── Task 5.6 ──────────────────────────────────────────────────────────────────
// Requirements: 2.3, 2.4
#[test]
fn test_revoke_pending_attestation() {
    let env = Env::default();
    env.mock_all_auths();

    let admin = Address::generate(&env);
    let issuer = Address::generate(&env);
    let subject = Address::generate(&env);
    let (_, client) = create_test_contract(&env);

    client.initialize(&admin);
    client.register_issuer(&admin, &issuer);

    let current_time: u64 = 1_000;
    env.ledger().with_mut(|l| l.timestamp = current_time);

    let future_time = current_time + 500;
    let claim_type = String::from_str(&env, "KYC_PASSED");

    let attestation_id =
        client.create_attestation(&issuer, &subject, &claim_type, &None, &Some(future_time));

    // Revoke while still pending
    client.revoke_attestation(&issuer, &attestation_id);

    // Time-lock is dominant: status is still Pending before valid_from
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Pending);

    // Advance ledger time past valid_from
    env.ledger().with_mut(|l| l.timestamp = future_time + 1);

    // Now the revocation takes effect: status is Revoked
    let status = client.get_attestation_status(&attestation_id);
    assert_eq!(status, types::AttestationStatus::Revoked);
}
