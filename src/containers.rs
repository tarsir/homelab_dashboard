use serde::Serialize;
use std::process::Command;

#[derive(Serialize)]
pub struct Container {
    id: String,
    ports: String,
    name: String,
    image: String,
}

impl Container {
    pub fn to_html_tr(&self) -> String {
        format!(
            "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
            self.name, self.ports, self.image, self.id
        )
    }
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
                    // id, ports, name, image
                    let parts = l.trim_matches('"').split(';').collect::<Vec<&str>>();
                    Some(Container {
                        id: parts[0].to_string(),
                        ports: parts[1].to_string(),
                        name: parts[2].to_string(),
                        image: parts[3].to_string(),
                    })
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
