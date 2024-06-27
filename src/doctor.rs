use clap::Args;

use crate::plugin::Plugin;
use crate::{busybox, dotnet, fix, git, kubernetes, mongo_db};

#[derive(Args)]
pub struct DoctorCommand {
    #[arg(short, long)]
    apply_fixes: bool,
}

pub struct DoctorSuccess {
    pub message: String,
    pub plugin: String,
}

pub struct DoctorFailure {
    pub message: String,
    pub plugin: String,
    pub fix: Option<Box<dyn Fn() -> Result<(), String>>>,
}

pub fn run(dr_command: DoctorCommand) {
    let plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(fix::Fix),
        Box::new(git::Git),
        Box::new(busybox::Busybox),
        Box::new(mongo_db::MongoDb),
        Box::new(kubernetes::Kubernetes),
        Box::new(dotnet::Dotnet),
    ];
    let mut results: Vec<(Result<DoctorSuccess, DoctorFailure>, Option<Result<(), String>>)> = Vec::new();

    for plugin in &plugins {
        let doctor_result = &mut plugin.doctor();
        let doctor_result_with_fixes: Vec<(Result<DoctorSuccess, DoctorFailure>, Option<Result<(), String>>)> = doctor_result
            .iter()
            .map(|r| 
                r.map(|o| (o, None))
                .map_err(|e| 
                        match dr_command.apply_fixes {
                            false => (e, None),
                            true => match e.fix {
                                None => (e, None),
                                Some(f) => (e, Some((f)()))
                            }
                        }))
            .collect();
        results.append(& mut doctor_result_with_fixes);
    }

    for result in results.iter() {
        match result {
            Ok(res) => {
                print!("✅ {}: {}\n", res.plugin, res.message)
            }
            Err(res) => {
                print!("❌ {}: {}\n", res.plugin, res.message)
            }
        }
    }

    for result in results.iter() {
        match result {
            Ok(res) => {
                print!("✅ {}: {}\n", res.plugin, res.message)
            }
            Err(res) if dr_command.apply_fixes == false => {
                print!("❌ {}: {}\n", res.plugin, res.message)
            }
            Err(res) if dr_command.apply_fixes == true => {
                print!("❌ {}: {}\n", res.plugin, res.message);
            }
            _ => {}
        }
    }
}
