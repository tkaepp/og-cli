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
                print!("✅ {}: {}\n", res.plugin, res.message)
            }
            Err(res) => match res {
                (x, Some(r)) => match r {
                    Ok(_) => print!("✅ Fixed {}: {}\n", x.plugin, x.message),
                    Err(y) => print!("❌ Could not fix {}: {} : {}\n", x.plugin, x.message, y),
                },
                (x, None) => print!("❌ {}: {}\n", x.plugin, res.0.message),
            },
        }
    }
}
