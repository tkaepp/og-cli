use clap::Args;
use std::process::Command;

#[cfg(feature = "git")]
use crate::git;
use crate::{dotnet, fix, kube, mongo_db, network, plugin::Plugin, sql};

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
        #[cfg(feature = "git")]
        Box::new(git::Git),
        Box::new(mongo_db::MongoDb),
        Box::new(sql::Sql),
        Box::new(kube::Kubernetes),
        Box::new(dotnet::Dotnet),
        Box::new(network::Network),
    ];

    let doc_res: Vec<Result<DoctorSuccess, DoctorFailure>> =
        plugins.iter().map(|p| p.doctor()).flatten().collect();
    let doctor_result_with_fixes: Vec<
        Result<&DoctorSuccess, (&DoctorFailure, Option<Result<(), String>>)>,
    > = doc_res
        .iter()
        .map(|r| {
            let x = r.as_ref().map_err(|e| match dr_command.apply_fixes {
                false => (e, None),
                true => match &e.fix {
                    None => (e, None),
                    Some(f) => (e, Some((f)())),
                },
            });
            x
        })
        .collect();

    for result in doctor_result_with_fixes.iter() {
        match result {
            Ok(res) => {
                println!("✅ {}: {}", res.plugin, res.message)
            }
            Err(res) => match res {
                (x, Some(r)) => match r {
                    Ok(_) => println!("✅ Fixed {}: {}", x.plugin, x.message),
                    Err(y) => println!("❌ Could not fix {}: {} : {}", x.plugin, x.message, y),
                },
                (x, None) => println!("❌ {}: {}", x.plugin, res.0.message),
            },
        }
    }
}
pub fn is_command_in_path(command: &str) -> Result<DoctorSuccess, DoctorFailure> {
    let res = match Command::new(command)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
    {
        Ok(_) => Ok(DoctorSuccess {
            message: "is installed".to_string(),
            plugin: command.to_string(),
        }),
        Err(_) => Err(DoctorFailure {
            message: format!(
                "tool {} is not available. Make sure it is in the PATH",
                command
            ),
            plugin: command.to_string(),
            fix: Some(Box::new(|| {
                println!("Please install");
                Err("Could not install automatically".into())
            })),
        }),
    };
    res
}
