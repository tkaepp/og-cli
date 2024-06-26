use crate::git;

pub struct Git;


impl Git {
    pub fn run(cli: git::commands::GitCommand) {
        println!("hello from git run");
    }
}
