use std::collections::HashSet;

use bincode::{Decode, Encode};

#[derive(Debug)]
pub enum MetadataTokens {
    Name(String),
    Version(String),
    Summary(String),
    Dependency(String),
}

#[derive(Debug, Clone, Encode, Decode)]
pub struct Metadata {
    pub name: String,
    pub version: String,
    pub summary: String,
    pub dependencies: Option<HashSet<String>>,
}

#[derive(Default)]
pub struct MetadataBuilder {
    pub name: Option<String>,
    pub version: Option<String>,
    pub summary: Option<String>,
    pub dependencies: Option<HashSet<String>>,
}

impl MetadataBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            version: None,
            summary: None,
            dependencies: None,
        }
    }
    pub fn name(&mut self, name: String) -> &mut Self {
        self.name = Some(name);
        self
    }
    pub fn version(&mut self, version: String) -> &mut Self {
        self.version = Some(version);
        self
    }
    pub fn summary(&mut self, summary: String) -> &mut Self {
        self.summary = Some(summary);
        self
    }
    pub fn add_dependencies(&mut self, dependencies: HashSet<String>) -> &mut Self {
        self.dependencies = Some(dependencies);
        self
    }
    pub fn build(&mut self) -> Metadata {
        Metadata {
            name: self.name.clone().unwrap_or_default(),
            version: self.version.clone().unwrap_or_default(),
            summary: self.summary.clone().unwrap_or_default(),
            dependencies: self.dependencies.clone(),
        }
    }
}

impl Metadata {
    pub fn parse_tokens(tokens: Vec<MetadataTokens>) -> color_eyre::Result<Metadata> {
        let mut builder = &mut MetadataBuilder::default();
        let mut dependencies: HashSet<String> = HashSet::new();
        for tok in tokens {
            match tok {
                MetadataTokens::Name(name) => builder = builder.name(name),
                MetadataTokens::Version(version) => builder = builder.version(version),
                MetadataTokens::Summary(summary) => builder = builder.summary(summary),
                MetadataTokens::Dependency(dep) => {
                    let dep_name = Self::parse_dependency(dep);
                    let _ = dependencies.insert(dep_name);
                }
            }
        }
        if !dependencies.is_empty() {
            builder.add_dependencies(dependencies);
        }
        // TODO: don't build yet (cus dependencies)
        let md = builder.build();
        Ok(md)
    }

    fn parse_dependency(dep: String) -> String {
        Self::split_at_separator(dep)
    }

    fn split_at_separator(dep: String) -> String {
        // TODO: to parse extras correct we gotta do it differently
        //             pytest>=7.3.2; extra == "test"
        // name -------^     ^ ^    ^ ^     ^  ^
        // version-sep ------+ |    | |     |  |
        // version ------------+    | |     |  |
        // separator ---------------+ |     |  |
        // extra ---------------------+     |  |
        // double equals token -------------+  |
        // extra feature flag -----------------+
        // so this is gonna be a bit more elaborate
        let mut split_index = 0;
        for (i, c) in dep.char_indices() {
            match c {
                '>' | '=' | ' ' | '\n' | ';' | '!' | '<' => {
                    split_index = i;
                    break;
                }
                _ => { /* consume */ }
            }
        }
        if split_index == 0 {
            split_index = dep.len();
        }

        dep.split_at(split_index).0.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Tests for Metadata
    #[test]
    fn test_split_at_separator() {
        assert_eq!(
            Metadata::split_at_separator("requests>=2.25.1".to_string()),
            "requests"
        );
        assert_eq!(
            Metadata::split_at_separator("opencv-python (>=4.5.5)".to_string()),
            "opencv-python"
        );
        assert_eq!(
            Metadata::split_at_separator("numpy==1.21.4".to_string()),
            "numpy"
        );
        assert_eq!(
            Metadata::split_at_separator("pandas < 2.0".to_string()),
            "pandas"
        );
        assert_eq!(
            Metadata::split_at_separator("scipy!=1.7.0".to_string()),
            "scipy"
        );
        assert_eq!(
            Metadata::split_at_separator("pytest ; extra == \"test\"".to_string()),
            "pytest"
        );
        assert_eq!(
            Metadata::split_at_separator("simplejson==3.* ; extra == \"test\"".to_string()),
            "simplejson"
        );
        assert_eq!(
            Metadata::split_at_separator("pycparser".to_string()),
            "pycparser"
        );
    }

    #[test]
    fn test_parse_dependency() {
        assert_eq!(
            Metadata::parse_dependency("requests>=2.25.1".to_string()),
            "requests"
        );
        assert_eq!(Metadata::parse_dependency("numpy".to_string()), "numpy");
    }

    #[test]
    fn test_parse_tokens() {
        let tokens = vec![
            MetadataTokens::Name("my-package".to_string()),
            MetadataTokens::Version("1.2.3".to_string()),
            MetadataTokens::Summary("This is a test.".to_string()),
            MetadataTokens::Dependency("requests>=2.0".to_string()),
            MetadataTokens::Dependency("click".to_string()),
        ];

        let metadata = Metadata::parse_tokens(tokens).unwrap();

        assert_eq!(metadata.name, "my-package");
        assert_eq!(metadata.version, "1.2.3");
        assert_eq!(metadata.summary, "This is a test.");

        let expected_deps: HashSet<String> = ["requests".to_string(), "click".to_string()]
            .iter()
            .cloned()
            .collect();
        assert_eq!(metadata.dependencies, Some(expected_deps));
    }

    #[test]
    fn test_parse_tokens_no_deps() {
        let tokens = vec![
            MetadataTokens::Name("simple-package".to_string()),
            MetadataTokens::Version("0.1.0".to_string()),
            MetadataTokens::Summary("A simple package.".to_string()),
        ];

        let metadata = Metadata::parse_tokens(tokens).unwrap();

        assert_eq!(metadata.name, "simple-package");
        assert_eq!(metadata.version, "0.1.0");
        assert_eq!(metadata.summary, "A simple package.");
        assert!(metadata.dependencies.is_none());
    }
}
