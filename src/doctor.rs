use clap::Args;

use crate::plugin::{DoctorFix, Plugin};
use crate::{dotnet, fix, git, kube, mongo_db, sql};

#[derive(Args, Debug)]
pub struct DoctorCommand {
    #[arg(short, long)]
    apply_fixes: bool,
}

pub struct DoctorSuccess {
    pub message: String,
    pub plugin: String,
}

#[derive(Debug)]
pub struct DoctorFailure {
    pub message: String,
    pub plugin: String,
}

pub fn run(dr_command: DoctorCommand) {
    match dr_command.apply_fixes {
        false => {}
        true => {
            println!("apply fixes on failing checks")
        }
    }

    let plugins_with_fixes: Vec<Box<dyn DoctorFix>> = vec![Box::new(git::Git)];

    let plugins: Vec<Box<dyn Plugin>> = vec![
        Box::new(fix::Fix),
        Box::new(mongo_db::MongoDb),
        Box::new(sql::Sql),
        Box::new(kube::Kubernetes),
        Box::new(dotnet::Dotnet),
    ];
    let mut results = Vec::new();

    for plugin in &plugins {
        results.append(&mut plugin.doctor());
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

    for plugin in &plugins_with_fixes {
        results.append(&mut plugin.doctor());
        plugin.apply_fix();
        results.append(&mut plugin.doctor());
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
