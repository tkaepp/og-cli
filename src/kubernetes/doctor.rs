use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::kubernetes::Kubernetes;
use crate::plugin::Plugin;

impl Plugin for Kubernetes {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        todo!()
    }
}