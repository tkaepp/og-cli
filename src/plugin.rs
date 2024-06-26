use crate::doctor::{DoctorFailure, DoctorSuccess};

pub trait Plugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>>;
}

pub  trait DoctorFix: Plugin{
    fn apply_fix(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>>;
}