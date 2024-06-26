use crate::doctor::{DoctorFailure, DoctorSuccess};
use crate::git::Git;
use crate::plugin::Plugin;

impl Plugin for Git {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        todo!()
    }
}
