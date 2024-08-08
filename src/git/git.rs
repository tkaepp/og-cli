use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use std::{ffi::OsStr, process::Command};

use dialoguer::MultiSelect;
use eyre::{eyre, ContextCompat, Result};
use homedir::get_my_home;
use log::info;
use ssh2_config::{ParseRule, SshConfig};
use ssh_key::{rand_core::OsRng, Algorithm, LineEnding, PrivateKey, PublicKey};

use crate::config;

use super::commands::{GitCommand, GitSubCommands};

pub struct GitPlugin;

impl GitPlugin {
    pub async fn run(cli: GitCommand) {
        match cli.command {
            GitSubCommands::Setup => setup().unwrap(),
            GitSubCommands::TailscaleAlias {
                user,
                identity_file,
                tag_filter,
                hostname_prefix,
            } => {
                which::which("tailscale").expect("tailscale must be installed");
                let hostnames = get_tailscale_hosts(tag_filter).await.unwrap();
                let filtered = hostnames.iter().filter(|h| {
                    if let Some(prefix) = &hostname_prefix {
                        return h.starts_with(prefix);
                    }
                    return true;
                });

                for hostname in filtered {
                    print_config_entry(
                        hostname.as_str(),
                        user.as_str(),
                        identity_file.to_str().unwrap(),
                    )
                }
            }
        }
    }
}

fn print_config_entry(hostname: &str, user: &str, ssh_identity_path: &str) {
    println!("Host {}", hostname);
    println!("\tUser {}", user);
    println!("\tIdentityFile {ssh_identity_path}",);
}

async fn get_tailscale_hosts(tag_filter: Option<String>) -> Result<Vec<String>> {
    let socket_path = "/var/run/tailscale/tailscaled.sock";
    let client = tailscale_localapi::LocalApi::new_with_socket_path(socket_path);
    let client_status = client.status().await?;

    let buildservers: Vec<String> = client_status
        .peer
        .iter()
        .filter(|(_, ps)| {
            if let Some(filter) = &tag_filter {
                return ps.tags.contains(filter);
            }
            return true;
        })
        .map(|(_, ps)| ps.hostname.clone())
        .collect();

    println!("{:?}", buildservers);
    Ok(buildservers)
}

fn setup() -> Result<()> {
    let _ = ensure_ssh_keys()?;

    add_keys_github()?;

    Ok(())
}

#[cfg(target_family = "unix")]
fn add_keys_github() -> Result<()> {
    Command::new("sh")
        .arg("-c")
        .arg("gh ssh-key add ~/.ssh/og-ssh.pub -t og")
        .output()
        .expect("failed to upload ssh key to github");

    Ok(())
}

#[cfg(target_family = "windows")]
fn add_keys_github() -> Result<()> {
    // Command::new("cmd")
    //     .args(["/C", "gh ssh-key add ~/.ssh/og-ssh.pub -t og"])
    //     .output()
    //     .expect("failed to execute process")
    todo!("implement ssh key upload on windows")
}

#[cfg(target_family = "unix")]
fn ensure_ssh_keys() -> Result<Vec<PublicKey>> {
    let ssh_dir = get_my_home()?
        .context("Could not get home directory")?
        .join(".ssh2"); // move this check to the doctor

    let public_keys: Vec<PublicKey> = ssh_dir
        .read_dir()?
        .filter_map(|r| r.ok())
        .map(|r| {
            (
                r.path()
                    .extension()
                    .and_then(OsStr::to_str)
                    .map(|s| s.to_string()),
                r,
            )
        })
        .filter(|(extension, _)| extension.as_ref().map(|c| c == "pub").unwrap_or(false))
        .map(|(_, e)| PublicKey::read_openssh_file(e.path().as_path()))
        .filter_map(|p| p.ok())
        .collect();

    if public_keys.is_empty() {
        let creation_options = &["Azure Devops", "Github"];
        let defaults = &[true, true];
        let selection = MultiSelect::new()
            .with_prompt("Should OG create ssh keys for you?")
            .items(&creation_options[..])
            .defaults(&defaults[..])
            .interact()
            .unwrap();

        if selection.is_empty() {
            return Err(eyre!("No git platforms were selected. Abort key creation"));
        }

        let private_key_ed = PrivateKey::random(&mut OsRng, Algorithm::Ed25519)?;

        private_key_ed
            .write_openssh_file(ssh_dir.join("og-ssh").as_path(), LineEnding::LF)
            .expect("key could not be created");
        private_key_ed
            .public_key()
            .write_openssh_file(ssh_dir.join("og-ssh.pub").as_path())
            .expect("key could not be created");

        return Ok(vec![private_key_ed.public_key().clone()]);
    } else {
        let selection = MultiSelect::new()
            .with_prompt("Select ssh keys to use")
            .items(&public_keys)
            .interact()
            .unwrap();
        info!("You chose:");

        return Ok(selection.iter().map(|r| public_keys[*r].clone()).collect());
    }
}

#[cfg(target_family = "windows")]
fn ensure_ssh_keys() -> Result<Vec<PublicKey>> {
    Ok(vec![])
}

#[cfg(target_family = "windows")]
fn check_ssh_keys() -> Result<()> {
    Ok(())
}
