use clap::Args;
use log::{error, info};
use which::which;

#[cfg(feature = "git")]
use crate::git;
use crate::{dotnet, fix, kube, mongo_db, network, plugin::Plugin, sql};

/// Detect and fix problems
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
        Box::new(fix::FixPlugin),
        #[cfg(feature = "git")]
        Box::new(git::GitPlugin),
        Box::new(mongo_db::MongoDbPlugin),
        Box::new(sql::SqlPlugin),
        Box::new(kube::KubernetesPlugin),
        Box::new(dotnet::DotnetPlugin),
        Box::new(network::NetworkPlugin),
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
                info!("✅ {}: {}", res.plugin, res.message)
            }
            Err(res) => match res {
                (x, Some(r)) => match r {
                    Ok(_) => error!("✅ Fixed {}: {}", x.plugin, x.message),
                    Err(y) => error!("❌ Could not fix {}: {} : {}", x.plugin, x.message, y),
                },
                (x, None) => error!("❌ {}: {}", x.plugin, res.0.message),
            },
        }
    }
}
pub fn is_command_in_path(command: &str) -> Result<DoctorSuccess, DoctorFailure> {
    let res = match which(command) {
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
                info!("Please install");
                Err("Could not install automatically".into())
            })),
        }),
    };
    res
}
