#![no_std]
use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, Address,
    Bytes, Env, Map, Symbol, Vec,
};

const ADMIN: Symbol = symbol_short!("admin");
const ONE_MONTH_LEDGERS: u32 = (30 * 24 * 60 * 60) / 5;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    CandidateNotFound = 1,
    VoterNotAuthorized = 2,
    VoterAlreadyVoted = 3,
    AdminNotAuthorized = 4,
    VotingEnded = 5,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Candidate {
    id: u64,
    name: Bytes,
    party: Bytes,
    avatar: Bytes,
    votes: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Voter {
    is_voted: bool,
}

pub fn get_candidates(env: &Env) -> Map<u64, Candidate> {
    let candidate_key = Symbol::new(&env, "candidates");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage()
        .instance()
        .get(&candidate_key)
        .unwrap_or(Map::new(&env))
}

pub fn set_candidates(env: &Env, candidates: &Map<u64, Candidate>) {
    let candidate_key = Symbol::new(&env, "candidates");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage().instance().set(&candidate_key, candidates);
}

pub fn get_authorized_voters(env: &Env) -> Map<Address, Voter> {
    let voters_key = Symbol::new(&env, "voters");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage()
        .instance()
        .get(&voters_key)
        .unwrap_or(Map::new(&env))
}

pub fn set_authorized_voters(env: &Env, voters: &Map<Address, Voter>) {
    let voters_key = Symbol::new(&env, "voters");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage().instance().set(&voters_key, voters);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage().instance().get(&ADMIN).unwrap()
}

pub fn set_admin(env: &Env, admin: &Address) {
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage().instance().set(&ADMIN, admin);
}

pub fn set_voted_status_voter(env: &Env, voter: Address) {
    let mut voters = get_authorized_voters(&env);
    voters.set(voter, Voter { is_voted: true });
    set_authorized_voters(&env, &voters);
}

pub fn increase_votes(env: &Env, id: u64) {
    let mut candidates = get_candidates(&env);
    let candidate = get_candidate(&env, id);
    candidates.set(
        id,
        Candidate {
            votes: candidate.votes + 1,
            ..candidate
        },
    );
    set_candidates(&env, &candidates);
}

pub fn get_candidate(env: &Env, id: u64) -> Candidate {
    let candidates = get_candidates(&env);
    let candidate = candidates.get(id);
    if candidate.is_none() {
        panic_with_error!(&env, Error::CandidateNotFound);
    }
    candidate.unwrap()
}

pub fn get_is_voting_active(env: &Env) -> bool {
    let candidate_key = Symbol::new(&env, "is_voting_active");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage()
        .instance()
        .get(&candidate_key)
        .unwrap_or(false)
}

pub fn set_is_voting_active(env: &Env, is_voting_active: bool) {
    let candidate_key = Symbol::new(&env, "is_voting_active");
    env.storage()
        .instance()
        .extend_ttl(ONE_MONTH_LEDGERS, ONE_MONTH_LEDGERS);
    env.storage()
        .instance()
        .set(&candidate_key, &is_voting_active);
}

#[contract]
pub struct VotingContract;

#[contractimpl]
impl VotingContract {
    pub fn __constructor(env: &Env, admin: Address) {
        set_admin(&env, &admin);
    }

    pub fn add_candidate(env: &Env, name: Bytes, party: Bytes, avatar: Bytes) -> u64 {
        get_admin(&env).require_auth();
        Self::check_voting_active(env);
        let mut candidates = get_candidates(&env);
        let id = env.prng().gen();
        let candidate = Candidate {
            id,
            name,
            party,
            avatar,
            votes: 0,
        };
        candidates.set(id, candidate);
        set_candidates(&env, &candidates);

        id
    }

    pub fn get_candidates_list(env: &Env) -> Vec<Candidate> {
        let candidates = get_candidates(&env);
        let mut result = Vec::new(&env);
        for (_, candidate) in candidates.iter() {
            result.push_back(candidate);
        }
        result
    }

    pub fn get_candidate(env: &Env, id: u64) -> Candidate {
        get_candidate(&env, id)
    }

    pub fn authorize_voter(env: &Env, voter: Address) {
        get_admin(&env).require_auth();
        Self::check_voting_active(env);
        let mut voters = get_authorized_voters(&env);
        voters.set(voter, Voter { is_voted: false });
        set_authorized_voters(&env, &voters);
    }

    pub fn revoke_voter(env: &Env, voter: Address) {
        get_admin(&env).require_auth();
        Self::check_voting_active(env);
        let mut voters = get_authorized_voters(&env);
        voters.remove(voter);
        set_authorized_voters(&env, &voters);
    }

    pub fn get_voter(env: &Env, voter: Address) -> Voter {
        let voters = get_authorized_voters(&env);
        let voter_data = voters.get(voter);
        if voter_data.is_none() {
            panic_with_error!(&env, Error::VoterNotAuthorized);
        }
        voter_data.unwrap()
    }

    pub fn vote(env: &Env, voter: Address, id: u64) {
        voter.require_auth();
        Self::check_voting_active(env);
        if Self::get_voter(env, voter.clone()).is_voted {
            panic_with_error!(&env, Error::VoterAlreadyVoted);
        }

        increase_votes(env, id);
        set_voted_status_voter(&env, voter.clone());

        env.events().publish((voter, symbol_short!("vote")), id);
    }

    pub fn get_total_votes(env: &Env) -> u32 {
        let candidates = get_candidates(&env);
        let mut total_votes = 0;
        for (_, candidate) in candidates.iter() {
            total_votes += candidate.votes;
        }
        total_votes
    }

    pub fn end_voting(env: &Env) {
        get_admin(&env).require_auth();
        set_is_voting_active(&env, false);
    }

    pub fn start_voting(env: &Env) {
        get_admin(&env).require_auth();
        if Self::get_total_votes(&env) > 0 {
            panic_with_error!(&env, Error::VotingEnded);
        }
        set_is_voting_active(&env, true);
    }

    pub fn check_voting_active(env: &Env) {
        if !get_is_voting_active(&env) {
            panic_with_error!(&env, Error::VotingEnded);
        }
    }

    // we use this function to get the winner(s) of the election in case of a tie between candidates with the same number of votes
    pub fn get_winners(env: &Env) -> Vec<Candidate> {
        let candidates = get_candidates(&env);
        let mut winners: Vec<Candidate> = Vec::new(&env);
        let mut highest_vote = 0;

        for (_, candidate) in candidates.iter() {
            if candidate.votes > highest_vote {
                highest_vote = candidate.votes;
                winners = Vec::new(&env);
                winners.push_back(candidate);
            } else if candidate.votes == highest_vote {
                winners.push_back(candidate);
            }
        }

        winners
    }
}

mod test;
