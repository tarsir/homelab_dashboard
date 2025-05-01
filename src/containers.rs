use serde::Serialize;
use std::{fmt::Display, process::Command};

#[derive(Serialize)]
pub struct Container {
    id: String,
    ports: std::vec::Vec<PortMapping>,
    name: String,
    image: String,
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
}

impl From<&str> for Container {
    fn from(value: &str) -> Self {
        // id, ports, name, image
        let parts = value.trim_matches('"').split(';').collect::<Vec<&str>>();
        let ports_list = parts[1]
            .split(',')
            .map(|p| p.trim())
            .filter(|p| p.contains('>'))
            .collect::<Vec<&str>>();
        println!("{:?}", ports_list);
        Container {
            id: parts[0].to_string(),
            ports: ports_list.into_iter().map(|p| p.into()).collect(),
            name: parts[2].to_string(),
            image: parts[3].to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
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
    fn to_html_list_item(&self) -> String {
        format!(
            "<li>{} -> {}, on {} ({})</li>",
            self.source_port, self.target_port, self.ip_addr, self.protocol
        )
    }
}

impl Display for PortMapping {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_html_list_item())
    }
}

impl From<&str> for PortMapping {
    fn from(value: &str) -> Self {
        let split_protocol = value.split('/').collect::<Vec<&str>>();
        let protocol: Protocol = split_protocol[1].into();
        let split_ip_addr = value.split(':').take(1).next().unwrap();
        let split_ports: Vec<&str> = value.split(&['/', ':', '>'][..]).skip(1).take(2).collect();
        println!("{:?}", split_ports);
        let source_port: i32 = split_ports[0]
            .trim_matches('-')
            .parse::<i32>()
            .unwrap_or(-1);
        let target_port: i32 = split_ports[1]
            .trim_matches('-')
            .parse::<i32>()
            .unwrap_or(-1);

        PortMapping {
            ip_addr: split_ip_addr.to_string(),
            source_port,
            target_port,
            protocol,
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
const DOCKER_PS_ARGS: &str = "ps --format \"{{.ID}};{{.Ports}};{{.Names}};{{.Image}}\"";

pub fn get_container_list() -> std::vec::Vec<Container> {
    let cmd_result = Command::new(DOCKER_PS_CMD)
        .args(DOCKER_PS_ARGS.split_whitespace())
        .output();
    if let Ok(output) = cmd_result {
        let output_lines = std::string::String::from_utf8(output.stdout);
        if let Ok(lines) = output_lines {
            lines
                .split('\n')
                .filter_map(|l: &str| {
                    if l.is_empty() {
                        return None;
                    }
                    Some(Container::from(l))
                })
                .collect()
        } else {
            println!("Failed to get container list");
            vec![]
        }
    } else {
        vec![]
    }
}
