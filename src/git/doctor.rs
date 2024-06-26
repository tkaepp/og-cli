use crate::git::Git;
use crate::plugin::Plugin;

impl Plugin for Git {
    fn doctor(&self) {
        println!("Check that you have a ssh key to use.");
    }
}
