> NOTE: The following is implementation examples for discussion on future models for DAOs

## User Data
```rust
pub struct ComposableUser {
  pub attestations: LookupMap<AttestationUuid, Vec<Attestation>>,
  pub attributions: LookupSet<Attribution>,
  // User approved list of accounts they allow to submit attestations or attributions on this account.
  // Default could be set with preferences, to either always allow new or always require approval
  pub notaries: LookupSet<AccountId>,
  // Allows user to control who & types of data allowed to be stored
  pub preferences: UnorderedMap<Hash, Vec<u8>>,
}
```


## Attestation Data
```rust
/// Attestations are signed data that can be proven by some type of cryptographic signature to verify its authenticity. This enforces portability for other entities to validate the source of some attestation.
/// REF: https://github.com/ethereum-attestation-service/contracts/blob/master/contracts/IEAS.sol
/// NOTE on EAS: Attestation schema is based on an attestation + registry system and does not allow for fluid data movement.
/// Additional fields below do not allow attestation registry or contract, but rather allow computed verification to live at or accessible to the entity proving the attestation.
/// This is done with a combination of a proof, signature type & inputs (can even be shared secret based or salted)
pub struct Attestation {
  // A unique identifier of the attestation.
  pub uuid: String,
  // A unique identifier of the AS.
  pub schema: String,
  // The recipient of the attestation.
  pub recipient: AccountId,
  // The attester/sender of the attestation.
  pub attester: AccountId,
  // The time when the attestation was created (Unix timestamp).
  pub time: u128,
  // The time when the attestation expires (Unix timestamp).
  pub expirationTime: Option<u128>,
  // The time when the attestation was revoked (Unix timestamp).
  pub revocationTime: Option<u128>,
  // The UUID of the related attestation. Used on delegated attestations
  pub refUUID: Option<u128>,
  // Custom attestation data.
  pub data: Option<Vec<u8>>,

  // Defines the cryptographic signature for validating the attestation proof, EX: ECDSA
  pub signature_scheme: String,
  // a value that can be computed using the specified signature engine in combination with attestation data
  pub proof: String,
  // additional context for any given attestation
  pub metadata: Option<Vec<u8>>,
}
```



## Attribution Data
```rust
/// Attributions are unsigned free-form data that allow entities to give where helpful. Think of this like a tag or nickname - a non-crucial piece of data, which can aid in usage of specific systems (EX: filtering).
pub struct Attribution {
  pub name: String,
  // The attester/sender of the attestation.
  pub attester: AccountId,
  // Custom attribution data.
  pub data: Option<Vec<u8>>,
  // additional context for any given attribution
  pub metadata: Option<Vec<u8>>,
}
```
