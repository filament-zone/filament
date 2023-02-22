use pulzaar_chain::Address;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Query {
    AccountByAddress(Address),
}
