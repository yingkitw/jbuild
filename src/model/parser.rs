use quick_xml::de::from_str;
use quick_xml::events::Event;
use quick_xml::Reader;
use thiserror::Error;

use crate::model::Model;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("XML parsing error: {0}")]
    Xml(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Invalid POM: {0}")]
    InvalidPom(String),
}

/// Parse a POM file from a string with namespace handling
pub fn parse_pom(pom_content: &str) -> Result<Model, ParseError> {
    // Maven POMs use namespaces like http://maven.apache.org/POM/4.0.0
    // We need to strip namespaces for quick-xml to work properly
    let normalized = normalize_xml_namespaces(pom_content)?;
    
    // Parse the normalized XML
    let model: Model = from_str(&normalized)
        .map_err(|e| ParseError::Xml(format!("XML deserialization error: {}", e)))?;
    
    // Validate required fields
    if model.group_id.is_empty() {
        return Err(ParseError::InvalidPom("groupId is required".to_string()));
    }
    if model.artifact_id.is_empty() {
        return Err(ParseError::InvalidPom("artifactId is required".to_string()));
    }
    if model.version.is_empty() {
        return Err(ParseError::InvalidPom("version is required".to_string()));
    }
    
    Ok(model)
}

/// Normalize XML by removing namespaces for easier parsing
pub fn normalize_xml_namespaces(xml: &str) -> Result<String, ParseError> {
    let mut reader = Reader::from_str(xml);
    reader.trim_text(true);
    
    let mut writer = Vec::new();
    let mut buf = Vec::new();
    
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(e)) => {
                let name = e.name();
                let name_str = String::from_utf8_lossy(name.as_ref());
                
                // Remove namespace prefix if present
                let local_name = if let Some(pos) = name_str.find(':') {
                    &name_str[pos + 1..]
                } else {
                    &name_str
                };
                
                // Write start tag without namespace
                writer.extend_from_slice(b"<");
                writer.extend_from_slice(local_name.as_bytes());
                
                // Copy attributes (also removing namespaces and xmlns declarations)
                for attr in e.attributes() {
                    if let Ok(attr) = attr {
                        let attr_name = String::from_utf8_lossy(attr.key.as_ref());
                        
                        // Skip xmlns and xsi namespace declarations
                        if attr_name == "xmlns" || attr_name.starts_with("xmlns:") || 
                           attr_name == "xsi:schemaLocation" || attr_name.starts_with("xsi:") {
                            continue;
                        }
                        
                        let local_attr_name = if let Some(pos) = attr_name.find(':') {
                            &attr_name[pos + 1..]
                        } else {
                            &attr_name
                        };
                        
                        writer.extend_from_slice(b" ");
                        writer.extend_from_slice(local_attr_name.as_bytes());
                        writer.extend_from_slice(b"=\"");
                        writer.extend_from_slice(&attr.value);
                        writer.extend_from_slice(b"\"");
                    }
                }
                
                writer.extend_from_slice(b">");
            }
            Ok(Event::End(e)) => {
                let name = e.name();
                let name_str = String::from_utf8_lossy(name.as_ref());
                let local_name = if let Some(pos) = name_str.find(':') {
                    &name_str[pos + 1..]
                } else {
                    &name_str
                };
                
                writer.extend_from_slice(b"</");
                writer.extend_from_slice(local_name.as_bytes());
                writer.extend_from_slice(b">");
            }
            Ok(Event::Text(e)) => {
                match e.unescape() {
                    Ok(text) => writer.extend_from_slice(text.as_bytes()),
                    Err(e) => return Err(ParseError::Xml(format!("XML unescape error: {}", e))),
                }
            }
            Ok(Event::Eof) => break,
            Ok(_) => {} // Ignore other events
            Err(e) => return Err(ParseError::Xml(format!("XML parsing error: {}", e))),
        }
        buf.clear();
    }
    
    String::from_utf8(writer)
        .map_err(|e| ParseError::InvalidPom(format!("Invalid UTF-8: {}", e)))
}

/// Parse a POM file from a file path
pub fn parse_pom_file(path: &std::path::Path) -> Result<Model, ParseError> {
    let content = std::fs::read_to_string(path)?;
    parse_pom(&content)
}
