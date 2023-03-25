//! Serde serialization for amount as msat

use cln_rpc::primitives::Amount;

pub trait SerdeAmount: Copy + Sized {
    fn from_msat(msats: u64) -> Self;
    fn as_msat(&self) -> u64;
}

impl SerdeAmount for Amount {
    fn from_msat(msats: u64) -> Self {
        Amount::from_msat(msats)
    }

    fn as_msat(&self) -> u64 {
        self.msat()
    }
}

pub mod as_msat {
    use super::SerdeAmount;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<A: SerdeAmount, S: Serializer>(a: &A, s: S) -> Result<S::Ok, S::Error> {
        a.as_msat().serialize(s)
    }

    pub fn deserialize<'d, A: SerdeAmount, D: Deserializer<'d>>(d: D) -> Result<A, D::Error> {
        let msats = u64::deserialize(d)?;
        Ok(A::from_msat(msats))
    }
}
