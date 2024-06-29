use std::process::Command;

use super::{doctor::Tools, NetworkCommand};
use crate::doctor::{is_command_in_path, DoctorFailure, DoctorSuccess};

pub struct Network;

impl Network {
    pub fn run(_: NetworkCommand) {
        println!("Running Network Tests");
        let ping_result =
            is_command_in_path(Tools::Nslookup.to_string().as_str()).and(Self::ping("10.1.4.4"));

        match ping_result {
            Ok(x) => {
                println!("✅  {}", x.message)
            }
            Err(x) => {
                println!("❌ {}", x.message)
            }
        }

        let nslookup_result = is_command_in_path(Tools::Nslookup.to_string().as_str())
            .and(Self::nslookup("digitec.ch", "10.1.4.4"))
            .and(Self::nslookup("backstage.devinite.com", "10.1.4.4"))
            .and(Self::nslookup(
                "kubernetes-management.int.devinite.com",
                "10.1.4.4",
            ))
            .and(Self::nslookup("sqld-az-vm01.intranet.digitec", "10.1.4.4"));

        match nslookup_result {
            Ok(x) => {
                println!("✅  {}", x.message)
            }
            Err(x) => {
                println!("❌ {}", x.message)
            }
        }
    }

    pub fn ping(address: &str) -> Result<DoctorSuccess, DoctorFailure> {
        Command::new("ping")
            .args(["-c 4", address])
            .output()
            .map(|_| DoctorSuccess {
                message: format!("ping {} succeeded", address,),
                plugin: "network-check".to_string(),
            })
            .map_err(|_| DoctorFailure {
                message: format!("nslookup {} failed", address),
                plugin: "network-check".to_string(),
                fix: None,
            })
    }
    pub fn nslookup(domain: &str, dns_server: &str) -> Result<DoctorSuccess, DoctorFailure> {
        Command::new("nslookup")
            .args([domain, dns_server])
            .output()
            .map(|_| DoctorSuccess {
                message: format!("nslookup {} {} succeeded", domain, dns_server),
                plugin: "network-check".to_string(),
            })
            .map_err(|_| DoctorFailure {
                message: format!("nslookup {} {} failed", domain, dns_server),
                plugin: "network-check".to_string(),
                fix: None,
            })
    }
}
