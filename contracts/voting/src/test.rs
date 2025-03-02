#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, vec, Env};

#[test]
fn test_authorized_voters() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();
    client.start_voting();

    let voter_1 = Address::generate(&env);
    let voter_2 = Address::generate(&env);

    client.authorize_voter(&voter_1);
    client.authorize_voter(&voter_2);

    assert!(!client.get_voter(&voter_1).is_voted);
    assert!(!client.get_voter(&voter_2).is_voted);
}

#[test]
fn test_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);
    let voter_2 = Address::generate(&env);

    client.authorize_voter(&voter_1);
    client.authorize_voter(&voter_2);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );

    client.vote(&voter_1, &candidate_id);
    assert!(client.get_voter(&voter_1).is_voted);
    assert_eq!(client.get_candidate(&candidate_id).votes, 1);

    client.vote(&voter_2, &candidate_id);
    assert!(client.get_voter(&voter_2).is_voted);
    assert_eq!(client.get_candidate(&candidate_id).votes, 2);

    assert_eq!(
        client.get_winners(),
        vec![&env, client.get_candidate(&candidate_id)]
    );

    // assert_eq!(
    //     env.events().all(),
    //     vec![
    //         &env,
    //         (
    //             contract_id.clone(),
    //             (voter_1, symbol_short!("vote")).into_val(&env),
    //             candidate_id.into_val(&env),
    //         ),
    //         (
    //             contract_id.clone(),
    //             (voter_2, symbol_short!("vote")).into_val(&env),
    //             candidate_id.into_val(&env),
    //         )
    //     ]
    // )
}

#[test]
fn test_get_winners() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);
    let voter_2 = Address::generate(&env);

    client.authorize_voter(&voter_1);
    client.authorize_voter(&voter_2);

    let candidate_id_1 = client.add_candidate(
        &Bytes::from_slice(&env, "candidate_1".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    let candidate_id_2 = client.add_candidate(
        &Bytes::from_slice(&env, "candidate_2".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );

    client.vote(&voter_1, &candidate_id_1);
    client.vote(&voter_2, &candidate_id_2);

    assert_eq!(
        client.get_winners(),
        vec![
            &env,
            client.get_candidate(&candidate_id_1),
            client.get_candidate(&candidate_id_2)
        ]
    );
}

#[test]
fn test_total_votes() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);
    let voter_2 = Address::generate(&env);

    client.authorize_voter(&voter_1);
    client.authorize_voter(&voter_2);

    let candidate_id_1 = client.add_candidate(
        &Bytes::from_slice(&env, "candidate_1".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    let candidate_id_2 = client.add_candidate(
        &Bytes::from_slice(&env, "candidate_2".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );

    client.vote(&voter_1, &candidate_id_1);
    assert!(client.get_voter(&voter_1).is_voted);
    assert_eq!(client.get_candidate(&candidate_id_1).votes, 1);

    client.vote(&voter_2, &candidate_id_2);
    assert!(client.get_voter(&voter_2).is_voted);
    assert_eq!(client.get_candidate(&candidate_id_2).votes, 1);

    assert_eq!(client.get_total_votes(), 2);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_voter_is_unauthorized() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );

    client.vote(&voter_1, &candidate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_voter_revoked() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    client.authorize_voter(&voter_1);
    client.revoke_voter(&voter_1);
    client.vote(&voter_1, &candidate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_vote_candidate_not_found() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);

    client.authorize_voter(&voter_1);
    client.vote(&voter_1, &1);
}

#[test]
#[should_panic(expected = "Error(Contract, #3)")]
fn test_duplicate_vote() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);
    client.authorize_voter(&voter_1);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    client.vote(&voter_1, &candidate_id);
    client.vote(&voter_1, &candidate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_vote_unstarted_voting() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);

    let voter_1 = Address::generate(&env);
    client.authorize_voter(&voter_1);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    client.vote(&voter_1, &candidate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_vote_ended_voting() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();
    client.end_voting();

    let voter_1 = Address::generate(&env);
    client.authorize_voter(&voter_1);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    client.vote(&voter_1, &candidate_id);
}

#[test]
#[should_panic(expected = "Error(Contract, #5)")]
fn test_restart_voting_when_votes_is_received() {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let contract_id = env.register(VotingContract, (&admin,));
    let client = VotingContractClient::new(&env, &contract_id);
    client.start_voting();

    let voter_1 = Address::generate(&env);
    let voter_2 = Address::generate(&env);
    client.authorize_voter(&voter_1);
    client.authorize_voter(&voter_2);

    let candidate_id = client.add_candidate(
        &Bytes::from_slice(&env, "candidate".as_bytes()),
        &Bytes::from_slice(&env, "party".as_bytes()),
        &Bytes::from_slice(&env, "avatar".as_bytes()),
    );
    client.vote(&voter_1, &candidate_id);
    client.end_voting();
    client.start_voting();
}
