use crate::{busybox, fix, mongo_db};
use crate::plugin::Plugin;

pub struct DoctorSuccess
{
    pub message: String,
    pub plugin: String
}

#[derive(Debug)]
pub struct DoctorFailure
{
    pub message: String,
    pub plugin: String
}

pub fn run() {
    let plugins: Vec<Box<dyn Plugin>> =
        vec![Box::new(fix::Fix), Box::new(busybox::Busybox), Box::new(mongo_db::MongoDb)];
    let mut results = Vec::new();
    for plugin in &plugins {
        results.append(&mut plugin.doctor());
    }

    for result in results.iter() {
        match result {
            Ok(res) => { print!("✅ {}: {}\n", res.plugin, res.message) },
            Err(res) => { print!("❌ {}: {}\n", res.plugin, res.message) }
        }
    }
}