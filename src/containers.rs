use serde::Serialize;
use std::{collections::HashSet, fmt::Display, path::Path, process::Command};

#[derive(Serialize)]
pub struct Container {
    id: String,
    ports: std::vec::Vec<PortMapping>,
    name: String,
    image: String,
    time_running: String,
}

impl Container {
    pub fn to_html_tr(&self) -> String {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            self.name,
            port_map_list_to_html_ul(&self.ports),
            self.image,
            self.id
        )
    }

    pub fn to_html_card(&self, host: &str) -> String {
        format!(
            r#"
<div class="bg-white rounded-lg shadow-md p-6 container-card">
    <div class="flex justify-between items-start mb-4">
        <h3 class="text-lg font-semibold text-gray-800">{}</h3>
        <span class="bg-green-100 text-green-800 text-xs font-medium px-2.5 py-0.5 rounded">Running since {}</span>
    </div>
    <div class="mb-4">
        <p class="text-sm text-gray-600 mb-1"><span class="font-medium">Image:</span> {}</p>
        <p class="text-sm text-gray-600"><span class="font-medium">ID:</span> {}</p>
    </div>
    <div>
        <p class="text-sm font-medium text-gray-700 mb-2">Ports:</p>
        {}
    </div>
</div>
"#,
            self.name,
            self.time_running,
            self.image,
            self.id,
            port_map_list_to_html_div(&self.ports, host)
        )
    }
}

// Add a new function to format port mappings as divs instead of ul
pub fn port_map_list_to_html_div(port_maps: &std::vec::Vec<PortMapping>, host: &str) -> String {
    if port_maps.is_empty() {
        return "<p class=\"text-sm text-gray-500\">No ports exposed</p>".to_string();
    }

    format!(
        "<div class=\"space-y-1\">{}{}</div>",
        port_maps
            .iter()
            .map(|pm| format!(
                r#"<a href="http://{}:{}"><div class="text-sm text-gray-600 bg-gray-50 px-2 py-1 rounded">{}</div></a>"#,
                host,
                pm.external_port(),
                pm
            ))
            .collect::<String>(),
        "</div>"
    )
}

impl From<&str> for Container {
    fn from(value: &str) -> Self {
        // id, ports, name, image
        let parts = value.trim_matches('"').split(';').collect::<Vec<&str>>();
        let ports_list = if parts.len() > 1 {
            parts[1]
                .split(',')
                .map(|p| p.trim())
                .filter(|p| !p.is_empty() && !p.starts_with(":::"))
                .collect::<Vec<&str>>()
        } else {
            vec![]
        };
        println!("parsed {:?} from string: {}", ports_list, value);
        Container {
            id: parts[0].to_string(),
            ports: ports_list.into_iter().map(|p| p.into()).collect(),
            name: parts[2].to_string(),
            image: parts[3].to_string(),
            time_running: parts[4].to_string(),
        }
    }
}

#[derive(Debug, Serialize, PartialEq)]
pub enum Protocol {
    Tcp,
    Udp,
    Empty,
    Other,
}

impl Display for Protocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Protocol::Tcp => write!(f, "TCP"),
            Protocol::Udp => write!(f, "UDP"),
            Protocol::Empty => write!(f, "Empty"),
            Protocol::Other => write!(f, "Other"),
        }
    }
}

impl From<&str> for Protocol {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_ref() {
            "tcp" => Protocol::Tcp,
            "udp" => Protocol::Udp,
            "" => Protocol::Empty,
            _ => Protocol::Other,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct PortMapping {
    ip_addr: String,
    source_port: i32,
    target_port: i32,
    protocol: Protocol,
}

impl PortMapping {
    pub fn external_port(&self) -> i32 {
        if self.source_port > 0 {
            self.source_port
        } else {
            self.target_port
        }
    }

    fn to_html_list_item(&self) -> String {
        format!(
            "<li>{} -> {}, on {} ({})</li>",
            self.source_port, self.target_port, self.ip_addr, self.protocol
        )
    }
}

impl Display for PortMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.source_port > 0 && self.source_port != self.target_port {
            write!(
                f,
                "{} -> {} ({})",
                self.source_port, self.target_port, self.protocol
            )
        } else {
            write!(f, "{} ({})", self.target_port, self.protocol)
        }
    }
}

impl From<&str> for PortMapping {
    fn from(value: &str) -> Self {
        let (mapping_part, protocol_str) = match value.split_once('/') {
            Some((m, proto)) => (m, proto),
            None => (value, ""),
        };
        let protocol: Protocol = Protocol::from(protocol_str);

        if let Some((host_part, target_part)) = mapping_part.split_once("->") {
            let target_port = target_part.parse::<i32>().unwrap_or(-1);
            let (ip_addr, source_port) = match host_part.rsplit_once(':') {
                Some((ip, port_str)) => (ip.to_string(), port_str.parse::<i32>().unwrap_or(-1)),
                None => (
                    "0.0.0.0".to_string(),
                    host_part.parse::<i32>().unwrap_or(-1),
                ),
            };

            PortMapping {
                ip_addr,
                source_port,
                target_port,
                protocol,
            }
        } else {
            let target_port = mapping_part.parse::<i32>().unwrap_or(-1);
            PortMapping {
                ip_addr: String::new(),
                source_port: 0,
                target_port,
                protocol,
            }
        }
    }
}

pub fn port_map_list_to_html_ul(port_maps: &std::vec::Vec<PortMapping>) -> String {
    format!(
        "<ul>{}</ul>",
        port_maps
            .iter()
            .flat_map(|pm| { format!("<li>{}</li>", pm).chars().collect::<Vec<char>>() })
            .collect::<String>()
    )
}

const DOCKER_PS_CMD: &str = "docker";
const DOCKER_PS_FORMAT: &str = "{{.ID}};{{.Ports}};{{.Names}};{{.Image}};{{.RunningFor}};";

pub fn get_container_list() -> std::vec::Vec<Container> {
    // Sockets to check for multi-socket setups (e.g. root + user Quadlets)
    let potential_sockets = ["/run/user-podman.sock", "/run/root-podman.sock"];
    let active_sockets: Vec<&str> = potential_sockets
        .into_iter()
        .filter(|sock| Path::new(sock).exists())
        .collect();

    // If specific multi-sockets exist, query each. Otherwise fall back to the default socket (/var/run/docker.sock).
    let socket_targets: Vec<Option<&str>> = if !active_sockets.is_empty() {
        active_sockets.into_iter().map(Some).collect()
    } else {
        vec![None]
    };

    let mut all_containers = Vec::new();
    let mut seen_ids = HashSet::new();

    for sock in socket_targets {
        let mut cmd = Command::new(DOCKER_PS_CMD);
        if let Some(sock_path) = sock {
            cmd.args(["-H", &format!("unix://{}", sock_path)]);
        }
        cmd.args(["ps", "--format", DOCKER_PS_FORMAT]);

        if let Ok(output) = cmd.output() {
            if let Ok(output_str) = String::from_utf8(output.stdout) {
                for line in output_str.lines() {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    let container = Container::from(trimmed);
                    if seen_ids.insert(container.id.clone()) {
                        all_containers.push(container);
                    }
                }
            } else {
                println!("Failed to parse output for socket {:?}", sock);
            }
        } else {
            println!("Failed to run docker ps for socket {:?}", sock);
        }
    }

    all_containers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_ports() {
        let test_port = "9090/tcp";
        let mapped_ports: PortMapping = test_port.into();
        assert_eq!(mapped_ports.external_port(), 9090);
        assert_eq!(mapped_ports.protocol, Protocol::Tcp);

        let mapped_full: PortMapping = "0.0.0.0:8080->80/tcp".into();
        assert_eq!(mapped_full.external_port(), 8080);
        assert_eq!(mapped_full.target_port, 80);
        assert_eq!(mapped_full.protocol, Protocol::Tcp);
    }

    #[test]
    fn parse_container_with_bare_ports() {
        let raw = "abc123;9090/tcp;my-service;my-image:latest;2 hours ago;";
        let container = Container::from(raw);
        assert_eq!(container.id, "abc123");
        assert_eq!(container.name, "my-service");
        assert_eq!(container.ports.len(), 1);
        assert_eq!(container.ports[0].external_port(), 9090);
    }
}
