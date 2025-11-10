use anyhow::{Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use std::io::Read;

/// Plugin descriptor
#[derive(Debug, Clone)]
pub struct PluginDescriptor {
    pub group_id: String,
    pub artifact_id: String,
    pub version: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub goal_prefix: Option<String>,
    pub goals: Vec<PluginGoal>,
    /// Plugin dependencies (from plugin POM)
    pub dependencies: Vec<crate::model::Dependency>,
}

#[derive(Debug, Clone)]
pub struct PluginGoal {
    pub name: String,
    pub phase: Option<String>,
    pub description: Option<String>,
}

impl PluginDescriptor {
    /// Parse plugin descriptor from XML content
    pub fn from_xml(xml_content: &str) -> Result<Self> {
        let mut reader = Reader::from_str(xml_content);
        reader.trim_text(true);

        let mut descriptor = PluginDescriptor {
            group_id: String::new(),
            artifact_id: String::new(),
            version: String::new(),
            name: None,
            description: None,
            goal_prefix: None,
            goals: Vec::new(),
            dependencies: Vec::new(),
        };

        let mut buf = Vec::new();
        let mut current_element: Option<String> = None;
        let mut current_goal: Option<PluginGoal> = None;
        let mut in_mojos = false;

        loop {
            match reader.read_event_into(&mut buf) {
                Ok(Event::Start(e)) => {
                    match e.name().as_ref() {
                        b"plugin" => {
                            // Root element
                        }
                        b"mojos" => {
                            in_mojos = true;
                        }
                        b"mojo" => {
                            current_goal = Some(PluginGoal {
                                name: String::new(),
                                phase: None,
                                description: None,
                            });
                        }
                        _ => {
                            current_element = Some(String::from_utf8_lossy(e.name().as_ref()).to_string());
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    let text = e.unescape().unwrap_or_default().to_string();
                    if let Some(ref element) = current_element {
                        if in_mojos {
                            if let Some(ref mut goal) = current_goal {
                                match element.as_str() {
                                    "goal" => goal.name = text,
                                    "phase" => goal.phase = Some(text),
                                    "description" => goal.description = Some(text),
                                    _ => {}
                                }
                            }
                        } else {
                            match element.as_str() {
                                "groupId" => descriptor.group_id = text,
                                "artifactId" => descriptor.artifact_id = text,
                                "version" => descriptor.version = text,
                                "name" => descriptor.name = Some(text),
                                "description" => descriptor.description = Some(text),
                                "goalPrefix" => descriptor.goal_prefix = Some(text),
                                _ => {}
                            }
                        }
                    }
                }
                Ok(Event::End(e)) => {
                    match e.name().as_ref() {
                        b"mojo" => {
                            if let Some(goal) = current_goal.take() {
                                descriptor.goals.push(goal);
                            }
                        }
                        b"mojos" => {
                            in_mojos = false;
                        }
                        _ => {
                            current_element = None;
                        }
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow::anyhow!("XML parse error: {}", e));
                }
                _ => {}
            }
            buf.clear();
        }

        if descriptor.group_id.is_empty() || descriptor.artifact_id.is_empty() || descriptor.version.is_empty() {
            return Err(anyhow::anyhow!("Invalid plugin descriptor: missing required fields"));
        }

        Ok(descriptor)
    }

    /// Extract plugin descriptor from a JAR file
    pub fn from_jar(jar_path: &std::path::Path) -> Result<Self> {
        let file = std::fs::File::open(jar_path)
            .with_context(|| format!("Failed to open JAR file: {:?}", jar_path))?;
        let mut archive = zip::ZipArchive::new(file)
            .with_context(|| format!("Failed to read ZIP archive: {:?}", jar_path))?;

        // Look for plugin.xml in META-INF/maven/
        let plugin_xml_path = "META-INF/maven/plugin.xml";
        
        let mut plugin_file = archive.by_name(plugin_xml_path)
            .with_context(|| format!("Plugin descriptor not found in JAR: {}", plugin_xml_path))?;

        let mut xml_content = String::new();
        plugin_file.read_to_string(&mut xml_content)
            .with_context(|| "Failed to read plugin.xml from JAR")?;

        Self::from_xml(&xml_content)
    }
}

