use crate::plugin_api::descriptor::PluginDescriptor;
use crate::plugin_api::mojo::Mojo;

/// Maven plugin
pub trait Plugin {
    /// Get the plugin descriptor
    fn descriptor(&self) -> &PluginDescriptor;

    /// Get a mojo by goal name
    fn get_mojo(&self, goal: &str) -> Option<Box<dyn Mojo>>;
}

