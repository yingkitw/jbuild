use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use anyhow::{Context, Result};

#[cfg(feature = "jni")]
use jni::{
    JavaVM, InitArgsBuilder, objects::JValue,
};

/// JVM manager for Java plugin execution
#[cfg(feature = "jni")]
pub struct JvmManager {
    vm: Arc<JavaVM>,
}

#[cfg(feature = "jni")]
impl JvmManager {
    /// Initialize JVM with classpath
    pub fn new(classpath: &str) -> Result<Self> {
        let jvm_args = InitArgsBuilder::new()
            .version(jni::JNIVersion::V8)
            .option(&format!("-Djava.class.path={}", classpath))
            .build()
            .context("Failed to build JVM init args")?;

        let vm = JavaVM::new(jvm_args)
            .context("Failed to create Java VM")?;

        Ok(Self {
            vm: Arc::new(vm),
        })
    }

    /// Get the Java VM
    pub fn vm(&self) -> &Arc<JavaVM> {
        &self.vm
    }
}

#[cfg(feature = "jni")]
/// Global JVM manager (singleton)
static JVM_MANAGER: Mutex<Option<Arc<JvmManager>>> = Mutex::new(None);

#[cfg(feature = "jni")]
/// Get or create the global JVM manager
pub fn get_jvm_manager(classpath: &str) -> Result<Arc<JvmManager>> {
    let mut manager = JVM_MANAGER.lock().unwrap();
    
    if let Some(ref mgr) = *manager {
        return Ok(mgr.clone());
    }

    let new_manager = Arc::new(JvmManager::new(classpath)?);
    *manager = Some(new_manager.clone());
    Ok(new_manager)
}

/// Java Mojo executor using JNI
#[cfg(feature = "jni")]
pub struct JniMojoExecutor {
    jvm_manager: Arc<JvmManager>,
    plugin_jar: PathBuf,
    classpath: String,
}

#[cfg(feature = "jni")]
impl JniMojoExecutor {
    pub fn new(plugin_jar: PathBuf, classpath: String) -> Result<Self> {
        // Add Maven plugin API to classpath
        let full_classpath = format!("{}:{}", classpath, Self::get_maven_api_classpath()?);
        
        let jvm_manager = get_jvm_manager(&full_classpath)
            .context("Failed to get JVM manager")?;

        Ok(Self {
            jvm_manager,
            plugin_jar,
            classpath,
        })
    }

    /// Get Maven plugin API classpath (from local repository)
    fn get_maven_api_classpath() -> Result<String> {
        use crate::artifact::repository::DefaultLocalRepository;
        let local_repo = DefaultLocalRepository::default();
        
        // Try to find maven-plugin-api in local repository
        // Default location: ~/.m2/repository/org/apache/maven/maven-plugin-api/3.x.x/maven-plugin-api-3.x.x.jar
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        let mut api_path = PathBuf::from(home);
        api_path.push(".m2");
        api_path.push("repository");
        api_path.push("org");
        api_path.push("apache");
        api_path.push("maven");
        api_path.push("maven-plugin-api");
        
        // Try common versions
        for version in &["3.9.0", "3.8.0", "3.6.0"] {
            let mut jar_path = api_path.clone();
            jar_path.push(version);
            jar_path.push(format!("maven-plugin-api-{}.jar", version));
            
            if jar_path.exists() {
                return Ok(jar_path.to_string_lossy().to_string());
            }
        }

        // If not found, return empty (will fail later with better error)
        Ok(String::new())
    }

    /// Execute a Mojo using JNI
    pub fn execute_mojo(
        &self,
        mojo_class_name: &str,
        _project_data: &serde_json::Value,
    ) -> Result<()> {
        let _env = self.jvm_manager.vm().attach_current_thread()
            .context("Failed to attach to JVM thread")?;

        // Load the Mojo class
        // For now, we'll use a helper Java class that can load and execute Mojos
        tracing::info!("Executing Mojo class: {} via JNI", mojo_class_name);
        
        // TODO: Implement full Mojo instantiation and execution
        // This requires:
        // 1. Creating a Java helper class that can load Mojos
        // 2. Passing project/session data
        // 3. Calling execute() method
        
        Ok(())
    }
}

