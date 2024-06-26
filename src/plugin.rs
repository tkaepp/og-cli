use crate::doctor::{DoctorFailure, DoctorSuccess};

pub trait Plugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>>;
}
