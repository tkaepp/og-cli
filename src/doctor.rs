use clap::Args;

use crate::plugin::Plugin;
use crate::{dotnet, fix, git, kube, mongo_db, sql};

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
        Box::new(mongo_db::MongoDb),
        Box::new(sql::Sql),
        Box::new(kube::Kubernetes),
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
