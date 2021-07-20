use crate::refactor::{
    collections::TypedUsize,
    sdk::api::{BytesVec, TofnFatal, TofnResult},
};
use tracing::{error, info, warn};

pub fn serialize<T: ?Sized>(value: &T) -> TofnResult<BytesVec>
where
    T: serde::Serialize,
{
    match bincode::serialize(value) {
        Ok(bytes) => Ok(bytes),
        Err(err) => {
            error!("serialization failure: {}", err.to_string());
            Err(TofnFatal)
        }
    }
}

pub fn log_fault_info<K>(me: TypedUsize<K>, faulter: TypedUsize<K>, fault: &str) {
    info!("party {} detect [{}] by {}", me, fault, faulter,);
}

pub fn log_fault_warn<K>(me: TypedUsize<K>, faulter: TypedUsize<K>, fault: &str) {
    warn!("party {} detect [{}] by {}", me, fault, faulter,);
}

pub fn log_accuse_warn<K>(me: TypedUsize<K>, faulter: TypedUsize<K>, fault: &str) {
    warn!("party {} accuse {} of [{}]", me, faulter, fault);
}