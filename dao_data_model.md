> NOTE: The following is implementation examples for discussion on future models for DAOs

## DAO Registry Example
```rust
// This is just extending the normal dao data, allowing registries to offload logic & state
pub struct DAOContract {  
  /// Governance modules are unowned math libraries that are only responsible for data computation. They follow a rigid definition governance spec/interface that the DAO can utilize within proposals.
  pub governance_modules: UnorderedSet<AccountId>,
  /// Extensions keep track of registry of DAO approved extensions (keys), where state requirements are wrapped in value - if specific data needs to remain within the DAO vs inside the extension. (Think of DAO preferences or pagination indexes)
  pub extension_modules: UnorderedMap<AccountId, Vec<u8>>,
  
  /// NOTE: Policy may need changes to support diff groups control over governance modules/extensions
}
```


## DAO Proposal Data
NOTE: this would utilize the "VersionedProposal" setup, adding a new version

NOTE: Need to consider items brought up in: https://github.com/near-daos/sputnik-dao-contract/issues/161

```rust
pub struct Proposal {
// ---- ORIGINAL UNCHANGING ----------------------------
    /// Original proposer.
    pub proposer: AccountId,
    /// Description of this proposal.
    pub description: String,
    /// Kind of proposal with relevant information.
    pub kind: ProposalKind,
    /// Current status of the proposal.
    pub status: ProposalStatus,
    /// Submission time (for voting period).
    pub submission_time: U64,
// ---- CHANGING PROPOSAL ----------------------------
    /// DEPRECATE: Count of votes per role per decision: yes / no / spam. (Can be safely deprecated upon all active proposals being finalized)
    pub vote_counts: HashMap<String, [Balance; 3]>,
    /// DEPRECATE: Map of who voted and how. (Can be safely deprecated upon all active proposals being finalized)
    pub votes: HashMap<AccountId, Vote>,
// ---- NEW PROPOSAL DATA ----------------------------
    /// The module used to compute data against
    pub governance_module: AccountId,
    /// Proofs are votes with a more discrete / flexible payload the math modules can utilize
    /// NOTE: Map allows for ballot edits up until proposal finalization time
    pub ballots: UnorderedMap<AccountId, Ballot>,
    /// Computed index-based tally totals
    pub outcome: Outcome,
    /// metadata, allowing extensions or other innovations
    /// NOTE: This should allow URIs to IPFS docs or other proposal data needed for governance decisions
    pub metadata: Option<Vec<u8>>,
}
```


## New Proposal Kinds
```rust
pub enum ProposalKind {
    /// Enable conviction weight decisions to trigger differing functions/actions, 
    /// This will still only execute 1 proposal kind, however it allows the members 
    /// to decide the path based on multiple choice (or similar)
    IndexTriggeredActions(Vec<ProposalKind>),
}
```


## Governance Module Interface
```rust
// no state please, math only
// no owners allowed, confirm no access keys before use
pub trait Governance {
  /// Implements a function that call tally a set of data, returning the collated value
  pub fn compute(&self, ballots: Base64VecU8) -> PromiseOrValue<Outcome>;
}
```


## Extension Module Interface
```rust
// allows state, can allow iterative compute triggered/paginated outside the vote cycles
// owners allowed but highly discouraged, confirm no access keys before use
// Extensions are awesome when additional state is needed for proposals.
// Great usage will allow things like a cyclical proposal which includes a cooldown period
// that allows a 7-day extra window (for example) which could incorporate arbitration to correct
// a poor outcome, given some failure of governance or allowing 3rd party audit/weight post-vote.
// It allows for governance to have a safety or insurance window for remediation
// State-channel-like implementations should be the core of this type of module, however it could
// get as complicated as handling active strategies on a DeFi protocol
pub trait Extension {
  /// All contextual data is submitted at proposal creation
  /// Pre-vote state
  pub fn prepare(&mut self, proposal_id: u128, config: Base64VecU8);
  /// Implements a function that call tally a set of data, returning the collated value, including multi-computed values from state
  /// Active-vote state
  pub fn compute(&self, ballots: Base64VecU8) -> PromiseOrValue<Outcome>;
  /// Post-processing & 
  /// post-vote & pre-finalization state
  pub fn fulfill(&mut self, proposal_id: u128, proposal: Proposal);
  /// External actions based on proposal outcome data, after fulfill handles any additional vote-period closure
  /// finalization state
  pub fn finalize(&mut self, proposal_id: u128, proposal: Proposal);
}
```


## Outcome Spec
```rust
pub enum OutcomeKind {
  // For tracking [VoteApprove, VoteReject, VoteRemove]
  StdVote,
  // For tracking signaling based on dynamic length indexes
  IndexWeighted,
}

pub struct Outcome {
  kind: OutcomeKind,

  // NOTE: Needs to support balance level numbers in case votes are token weighted
  // NOTE: Index is critical for computing tallies
  /// flexible tallies computed by external module
  totals: Vec<u128>,

  /// Starts empty, until minimum threshold ballots is available
  finality_index: Option<usize>,
}

/// Examples:
// v2 voting style, where totals is tallies for: [VoteApprove, VoteReject, VoteRemove]
let vote_outcome = Outcome {
  kind: OutcomeKind::StdVote,
  totals: vec![12, 4, 1],
  finality_index: None,
};
// votes are counted at indexes, can utilize many governance modules to arrive at these counts
let indexed_outcome = Outcome {
  kind: OutcomeKind::IndexWeighted,
  totals: vec![1, 3, 19, 2],
  finality_index: Some(2),
};
```


## Ballot Data Example
```rust
pub struct Ballot {
  // NOTE: Needs to support balance level numbers in case votes are token weighted
  // NOTE: Index is critical for computing tallies
  weights: Vec<u128>,

  // Generic data
  // Allows for special ballot features like commit+reveal payloads
  // Data will ONLY be interpretable by the governance module
  data: Option<Base64VecU8>
}
```

# Module Examples

## Governance Module Demo: Simple Math Voting
```rust
struct Governance {}

impl Governance {
  /// Implements a function that call tally a set of data, returning the collated value
  // NOTE: this is pseudo-code - completely untested note-style code for reference of ideas
  pub fn compute(&self, ballots: &mut Base64VecU8) -> PromiseOrValue<Outcome> {
    // Extract the ballots
    let all_ballots: Ballot = serde_json::de::from_slice(&ballots).expect("Bad thing");
    let mut outcome = Outcome::default();

    // Loop and tally 
    all_ballots.iter().map(|(p, i)| {
      // NOTE: This is where things can get completely custom.
      // Could just do a simple tally, addition, multiplication - it is up to implementation requirements
      outcome.totals[i] += p[i];
    });

    // Compute latest winning index
    let max: u128 = outcome.totals.iter().max();
    outcome.winning_index = Some(outcome.totals.iter().position(|&x| x == max));

    PromiseOrValue::Value(Base64VecU8::from(outcome.to_string()))
  };
}
```

## Extension Module Demo: Simple Math Voting
```rust
struct Extension {}

impl Extension {
  /// All contextual data is submitted at proposal creation
  /// Pre-vote state
  pub fn prepare(&mut self, proposal_id: u128, config: Base64VecU8) {
    // TODO:
  };
  /// Implements a function that call tally a set of data, returning the collated value
  // NOTE: this is pseudo-code - completely untested note-style code for reference of ideas
  pub fn compute(&self, ballots: &mut Base64VecU8) -> PromiseOrValue<Outcome> {
    // Extract the ballots
    let all_ballots: Ballot = serde_json::de::from_slice(&ballots).expect("Bad thing");
    let mut outcome = Outcome::default();

    // Loop and tally 
    all_ballots.iter().map(|(p, i)| {
      // NOTE: This is where things can get completely custom.
      // Could just do a simple tally, addition, multiplication - it is up to implementation requirements
      outcome.totals[i] += p[i];
    });

    // Compute latest winning index
    let max: u128 = outcome.totals.iter().max();
    outcome.winning_index = Some(outcome.totals.iter().position(|&x| x == max));

    PromiseOrValue::Value(Base64VecU8::from(outcome.to_string()))
  };
  /// Post-processing & 
  /// post-vote & pre-finalization state
  pub fn fulfill(&mut self, proposal_id: u128, proposal: Proposal) {
    // TODO:
  };
  /// External actions based on proposal outcome data, after fulfill handles any additional vote-period closure
  /// finalization state
  pub fn finalize(&mut self, proposal_id: u128, proposal: Proposal) {
    // TODO:
  };
}
```
