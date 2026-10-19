use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize, Clone)]
pub struct Service {
    pub name: String,
    pub url: String,
    pub description: Option<String>,
    pub category: Option<String>,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct ServicesConfig {
    #[serde(default)]
    pub services: Vec<Service>,
}

const SERVICES_FILE: &str = "services.toml";

const DEFAULT_SERVICES_TOML: &str = r#"# Manual services / links configuration
# Services defined here will appear on your homelab dashboard.

[[services]]
name = "File Storage"
url = "https://files.homelab.lan"
description = "Internal Nextcloud / file server"
category = "Storage"

[[services]]
name = "Router & Firewall"
url = "https://router.homelab.lan"
description = "OPNsense / pfSense gateway"
category = "Network"

[[services]]
name = "Home Assistant"
url = "http://homeassistant.homelab.lan:8123"
description = "Home automation and IoT"
category = "Automation"
"#;

pub fn get_services() -> Vec<Service> {
    let path = Path::new(SERVICES_FILE);
    if !path.exists() {
        let _ = fs::write(path, DEFAULT_SERVICES_TOML);
    }

    match fs::read_to_string(path) {
        Ok(contents) => match toml::from_str::<ServicesConfig>(&contents) {
            Ok(config) => config.services,
            Err(e) => {
                eprintln!("Failed to parse {}: {}", SERVICES_FILE, e);
                vec![]
            }
        },
        Err(e) => {
            eprintln!("Failed to read {}: {}", SERVICES_FILE, e);
            vec![]
        }
    }
}

fn render_card(s: &Service) -> String {
    let desc_html = match &s.description {
        Some(desc) if !desc.trim().is_empty() => {
            format!(r#"<p class="text-sm text-gray-600 mb-1">{}</p>"#, desc)
        }
        _ => String::new(),
    };

    format!(
        r#"<a href="{url}" target="_blank" rel="noopener noreferrer" class="bg-white rounded-lg shadow-md p-3 container-card flex flex-col justify-between group hover:shadow-lg transition">
    <div>
        <div class="flex justify-between items-start mb-1 gap-2">
            <h3 class="text-lg font-semibold text-gray-800 group-hover:text-indigo-600 transition flex items-center gap-1.5">
                <span>{name}</span>
                <span class="text-gray-400 group-hover:text-indigo-600 text-sm">&nearr;</span>
            </h3>
        </div>
        {desc_html}
    </div>
    <div class="pt-1 border-t border-gray-100">
        <span class="text-xs font-mono text-indigo-600 hover:underline break-all">{url}</span>
    </div>
</a>"#,
        name = s.name,
        url = s.url,
        desc_html = desc_html,
    )
}

pub fn render_services_html() -> String {
    let services = get_services();
    render_services_from_list(&services)
}

pub fn render_services_from_list(services: &[Service]) -> String {
    if services.is_empty() {
        return r#"<div class="bg-white rounded-lg border border-dashed border-gray-300 p-6 text-center text-gray-500">
    No services configured yet. Add them to <code class="bg-gray-100 px-1 py-0.5 rounded font-mono text-sm">services.toml</code>.
</div>"#
            .to_string();
    }

    let has_explicit_categories = services
        .iter()
        .any(|s| s.category.as_deref().is_some_and(|c| !c.trim().is_empty()));

    if !has_explicit_categories {
        let cards = services.iter().map(render_card).collect::<String>();
        return format!(
            r#"<div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">{}</div>"#,
            cards
        );
    }

    let mut categories: Vec<(String, Vec<&Service>)> = Vec::new();
    for service in services {
        let cat_name = service
            .category
            .as_deref()
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .unwrap_or("General");

        if let Some((_, list)) = categories.iter_mut().find(|(name, _)| name == cat_name) {
            list.push(service);
        } else {
            categories.push((cat_name.to_string(), vec![service]));
        }
    }

    categories
        .iter()
        .map(|(cat_name, cat_services)| {
            let cards = cat_services.iter().map(|s| render_card(s)).collect::<String>();
            format!(
                r#"<div class="p-4 rounded-sm space-y-3 bg-indigo-200">
    <div class="flex items-center space-x-2 border-b border-gray-200 pb-2">
        <h3 class="text-lg font-semibold text-gray-500 uppercase tracking-wider">{}</h3>
        <span class="bg-indigo-50 text-indigo-700 text-md px-2 py-0.5 rounded-full font-medium">{}</span>
    </div>
    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
        {}
    </div>
</div>"#,
                cat_name,
                cat_services.len(),
                cards
            )
        })
        .collect::<String>()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_services_toml() {
        let sample_toml = r#"
        [[services]]
        name = "Files"
        url = "https://files.homelab.lan"
        description = "My storage"
        category = "Storage"
        "#;

        let config: ServicesConfig = toml::from_str(sample_toml).unwrap();
        assert_eq!(config.services.len(), 1);
        assert_eq!(config.services[0].name, "Files");
        assert_eq!(config.services[0].url, "https://files.homelab.lan");
        assert_eq!(config.services[0].category.as_deref(), Some("Storage"));
    }

    #[test]
    fn test_render_card() {
        let service = Service {
            name: "Test".to_string(),
            url: "https://example.com".to_string(),
            description: Some("Description".to_string()),
            category: Some("Network".to_string()),
        };
        let card_html = render_card(&service);
        assert!(card_html.contains("Test"));
        assert!(card_html.contains("https://example.com"));
        assert!(card_html.contains("Description"));
    }

    #[test]
    fn test_render_services_grouped() {
        let services = vec![
            Service {
                name: "Files".to_string(),
                url: "https://files.homelab.lan".to_string(),
                description: None,
                category: Some("Storage".to_string()),
            },
            Service {
                name: "Router".to_string(),
                url: "https://router.homelab.lan".to_string(),
                description: None,
                category: Some("Network".to_string()),
            },
            Service {
                name: "Uncategorized".to_string(),
                url: "https://example.com".to_string(),
                description: None,
                category: None,
            },
        ];

        let html = render_services_from_list(&services);
        assert!(html.contains("Storage"));
        assert!(html.contains("Network"));
        assert!(html.contains("General"));
    }
}
