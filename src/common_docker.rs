use std::collections::HashMap;
use std::io::Write;
use std::process::{self, Command};
use tempfile::NamedTempFile;

#[derive(Debug)]
pub struct DockerCompose {
    compose_file: NamedTempFile,
}

impl DockerCompose {
    pub fn new(compose_string: String) -> DockerCompose {
        let mut file = NamedTempFile::new().unwrap_or_else(|err| {
            println!("Could not create a temp file{}", err);
            process::exit(1)
        });
        write!(file, "{}", compose_string).unwrap_or_else(|err| {
            println!("{err}");
            process::exit(1)
        });

        DockerCompose { compose_file: file }
    }

    pub fn start(&self) {
        let command = Command::new("docker")
            .args([
                "compose",
                "-p",
                "test",
                "-f",
                (self.compose_file.path().to_str().unwrap()),
                "up",
                "-d",
            ])
            .output()
            .expect("Failed to run docker compose");

        let stdout = String::from_utf8(command.stdout).unwrap();
        dbg!(&self);
        println!("{}", &stdout);
    }

    pub fn stop(&self) {
        let command = Command::new("docker")
            .args([
                "compose",
                "-p",
                "test",
                "-f",
                (self.compose_file.path().to_str().unwrap()),
                "down",
            ])
            .output()
            .expect("Failed to stop docker compose");

        let stdout = String::from_utf8(command.stdout).unwrap();
        println!("{}", &stdout);
    }

    pub fn is_running() -> bool {
        let command = Command::new("docker")
            .args(["info"])
            .output();
        
        let stderr = String::from_utf8(command.unwrap().stderr).unwrap();

        if stderr.contains("Cannot connect to the Docker daemon") {
            return false;
        }

        return true;
    }
}

#[derive(Debug)]
pub struct DockerComposeBuilder {
    services: Vec<Service>,
}

impl DockerComposeBuilder {
    pub fn new() -> DockerComposeBuilder {
        DockerComposeBuilder {
            services: Vec::new(),
        }
    }

    pub fn add_service<S: Into<String>>(
        mut self,
        name: S,
        image: S,
        command: Option<S>,
        environment: Option<HashMap<String, String>>,
        port_mapping: Option<HashMap<u32, u32>>,
        volumes: Option<Vec<Volume>>,
    ) -> DockerComposeBuilder {
        self.services.push(Service {
            name: name.into(),
            image: image.into(),
            command: command.map(|s| s.into()),
            environment,
            port_mapping,
            volumes,
        });
        self
    }

    pub fn build(&self) -> DockerCompose {
        let mut sb = String::new();
        sb.push_str("services:\n");
        for service in &self.services {
            sb.push_str(&Self::build_service(service));
            sb.push('\n');
        }
        sb.push_str("volumes:\n");
        for service in &self.services {
            let volumes = &service.volumes;
            match &volumes {
                Some(volumes) => {
                    for volume in volumes.iter() {
                        sb.push_str(&*indent(format!("{}:", volume.volume_name)));
                    }
                }
                None => (),
            }
        }
        DockerCompose::new(sb)
    }

    pub fn build_string(&self) -> String {
        let mut sb = String::new();
        sb.push_str("service:\n");
        for service in &self.services {
            sb.push_str(&Self::build_service(service));
            sb.push('\n');
        }
        sb
    }

    fn build_service(service: &Service) -> String {
        let mut service_string = Vec::new();
        for x in service.build() {
            service_string.push(indent(x));
        }
        service_string.join("\n")
    }
}

#[derive(Debug)]
pub struct Service {
    name: String,
    image: String,
    command: Option<String>,
    environment: Option<HashMap<String, String>>,
    port_mapping: Option<HashMap<u32, u32>>,
    volumes: Option<Vec<Volume>>,
}

#[derive(Debug)]
pub struct Volume {
    pub volume_name: String,
    pub volume_type: String,
    pub bind: String,
    pub mode: String,
}

impl Service {
    pub fn build(&self) -> Vec<String> {
        let mut service_vec = Vec::new();
        service_vec.push(format!("{}:", self.name));
        service_vec.push(indent(format!("image: {}", self.image)));
        match &self.command {
            Some(command) => service_vec.push(indent(format!("command: {}", command))),
            None => (),
        }

        match &self.environment {
            Some(environments) => {
                service_vec.push(indent(String::from("environment:")));
                for (key, value) in environments.iter() {
                    service_vec.push(indent(indent(format!("- {}={}", key, value))));
                }
            }
            None => (),
        }

        match &self.port_mapping {
            Some(ports) => {
                service_vec.push(indent(String::from("ports:")));
                for (source, destination) in ports.iter() {
                    service_vec.push(indent(indent(format!("- \"{}:{}\"", source, destination))));
                }
            }
            None => (),
        }

        match &self.volumes {
            Some(volumes) => {
                service_vec.push(indent(String::from("volumes:")));
                for volume in volumes.iter() {
                    service_vec.push(indent(format!("- {}:{}", volume.volume_name, volume.bind)))
                }
            }
            None => (),
        }
        service_vec
    }
}

fn indent(string: String) -> String {
    format!("  {string}")
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn build_simple_service() {
        let dockerbuilder =
            DockerComposeBuilder::new().add_service("test", "testimage", None, None, None, None);
        dbg!(dockerbuilder.build_string());
        assert_eq!(
            dockerbuilder.build_string(),
            "service:\n  test:\n    image: testimage\n"
        )
    }

    #[test]
    fn build_multiple_service() {
        let dockerbuilder = DockerComposeBuilder::new()
            .add_service("test", "testimage", None, None, None, None)
            .add_service("test2", "testimage3", None, None, None, None);
        dbg!(dockerbuilder.build_string());
        assert_eq!(
            dockerbuilder.build_string(),
            "service:\n  test:\n    image: testimage\n  test2:\n    image: testimage3\n"
        )
    }
}
