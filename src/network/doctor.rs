use std::fmt::{Display, Formatter};

use super::NetworkPlugin;
use crate::{
    doctor::{is_command_in_path, DoctorFailure, DoctorSuccess},
    plugin::Plugin,
};

pub enum Tools {
    Nslookup,
    Nmap,
    Ping,
}

impl Display for Tools {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Tools::Nslookup => {
                write!(f, "nslookup")
            }
            Tools::Nmap => {
                write!(f, "nmap")
            }
            Tools::Ping => {
                write!(f, "ping")
            }
        }
    }
}

impl Plugin for NetworkPlugin {
    fn doctor(&self) -> Vec<Result<DoctorSuccess, DoctorFailure>> {
        let required_tools = vec![
            is_command_in_path(Tools::Nslookup.to_string().as_str()),
            is_command_in_path(Tools::Nmap.to_string().as_str()),
            is_command_in_path(Tools::Ping.to_string().as_str()),
        ];

        required_tools
    }
}
