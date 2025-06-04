use candid::CandidType;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize, CandidType, PartialEq, Eq)]
pub struct User {
    pub id: u64,
    pub name: String,
    pub age: u64,
}
