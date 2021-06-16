#![allow(clippy::needless_doctest_main)]
//!`system-deps` lets you write system dependencies in `Cargo.toml` metadata,
//! rather than programmatically in `build.rs`. This makes those dependencies
//! declarative, so other tools can read them as well.
//!
//! # Usage
//! In your `Cargo.toml`:
//!
//! ```toml
//! [build-dependencies]
//! system-deps = "2.0"
//! ```
//!
//! Then, to declare a dependency on `testlib >= 1.2`
//! add the following section:
//!
//! ```toml
//! [package.metadata.system-deps]
//! testlib = "1.2"
//! ```
//!
//! Finally, in your `build.rs`, add:
//!
//! ```should_panic
//! fn main() {
//!     system_deps::Config::new().probe().unwrap();
//! }
//! ```
//!
//! # Feature-specific dependency
//! You can easily declare an optional system dependency by associating it with a feature:
//!
//! ```toml
//! [package.metadata.system-deps]
//! testdata = { version = "4.5", feature = "use-testdata" }
//! ```
//!
//! `system-deps` will check for `testdata` only if the `use-testdata` feature has been enabled.
//!
//! # Optional dependency
//!
//! Another option is to use the `optional` setting, which can also be used using [features versions](#feature-versions):
//!
//! ```toml
//! [package.metadata.system-deps]
//! test-data = { version = "4.5", optional = true }
//! testmore = { version = "2", v3 = { version = "3.0", optional = true }}
//! ```
//!
//! `system-deps` will automatically export for each dependency a feature `system_deps_have_$DEP` where `$DEP`
//! is the `toml` key defining the dependency in [snake_case](https://en.wikipedia.org/wiki/Snake_case).
//! This can be used to check if an optional dependency has been found or not:
//!
//! ```
//! #[cfg(system_deps_have_testdata)]
//! println!("found test-data");
//! ```
//!
//! # Overriding library name
//! `toml` keys cannot contain dot characters so if your library name does you can define it using the `name` field:
//!
//! ```toml
//! [package.metadata.system-deps]
//! glib = { name = "glib-2.0", version = "2.64" }
//! ```
//! # Feature versions
//! `-sys` crates willing to support various versions of their underlying system libraries
//! can use features to control the version of the dependency required.
//! `system-deps` will pick the highest version among enabled features.
//! Such version features must use the pattern `v1_0`, `v1_2`, etc.
//!
//! ```toml
//! [features]
//! v1_2 = []
//! v1_4 = ["v1_2"]
//! v1_6 = ["v1_4"]
//!
//! [package.metadata.system-deps.gstreamer_1_0]
//! name = "gstreamer-1.0"
//! version = "1.0"
//! v1_2 = { version = "1.2" }
//! v1_4 = { version = "1.4" }
//! v1_6 = { version = "1.6" }
//! ```
//!
//! The same mechanism can be used to require a different library name depending on the version:
//!
//! ```toml
//! [package.metadata.system-deps.gst_gl]
//! name = "gstreamer-gl-1.0"
//! version = "1.14"
//! v1_18 = { version = "1.18", name = "gstreamer-gl-egl-1.0" }
//! ```
//!
//! # Target specific dependencies
//!
//! You can define target specific dependencies:
//!
//! ```toml
//! [package.metadata.system-deps.'cfg(target_os = "linux")']
//! testdata = "1"
//! [package.metadata.system-deps.'cfg(not(target_os = "macos"))']
//! testlib = "1"
//! [package.metadata.system-deps.'cfg(unix)']
//! testanotherlib = { version = "1", optional = true }
//! ```
//!
//! See [the Rust documentation](https://doc.rust-lang.org/reference/conditional-compilation.html)
//! for the exact syntax.
//! Currently those keys are supported:
//! - `target_arch`
//! - `target_endian`
//! - `target_env`
//! - `target_family`
//! - `target_os`
//! - `target_pointer_width`
//! - `target_vendor`
//! - `unix` and `windows`
//!
//! # Overriding build flags
//! By default `system-deps` automatically defines the required build flags for each dependency using the information fetched from `pkg-config`.
//! These flags can be overriden using environment variables if needed:
//! - `SYSTEM_DEPS_$NAME_SEARCH_NATIVE` to override the [`cargo:rustc-link-search=native`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-link-searchkindpath) flag;
//! - `SYSTEM_DEPS_$NAME_SEARCH_FRAMEWORK` to override the [`cargo:rustc-link-search=framework`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-link-searchkindpath) flag;
//! - `SYSTEM_DEPS_$NAME_LIB` to override the [`cargo:rustc-link-lib`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib) flag;
//! - `SYSTEM_DEPS_$NAME_LIB_FRAMEWORK` to override the [`cargo:rustc-link-lib=framework`](https://doc.rust-lang.org/cargo/reference/build-scripts.html#rustc-link-lib) flag;
//! - `SYSTEM_DEPS_$NAME_INCLUDE` to override the [`cargo:include`](https://kornel.ski/rust-sys-crate#headers) flag.
//!
//! With `$NAME` being the upper case name of the key defining the dependency in `Cargo.toml`.
//! For example `SYSTEM_DEPS_TESTLIB_SEARCH_NATIVE=/opt/lib` could be used to override a dependency named `testlib`.
//!
//! One can also define the environment variable `SYSTEM_DEPS_$NAME_NO_PKG_CONFIG` to fully disable `pkg-config` lookup
//! for the given dependency. In this case at least SYSTEM_DEPS_$NAME_LIB or SYSTEM_DEPS_$NAME_LIB_FRAMEWORK should be defined as well.
//!
//! # Statically build system library
//! `-sys` crates can provide support for building and statically link their underlying system library as part of their build process.
//! Here is how to do this in your `build.rs`:
//! ```should_panic
//! fn main() {
//!     system_deps::Config::new()
//!         .add_build_internal("testlib", |lib, version| {
//!             // Actually build the library here
//!             system_deps::Library::from_internal_pkg_config("build/path-to-pc-file", lib, version)
//!          })
//!         .probe()
//!         .unwrap();
//! }
//! ```
//!
//! This feature can be controlled using the `SYSTEM_DEPS_$NAME_BUILD_INTERNAL` environment variable
//! which can have the following values:
//! - `auto`: build the dependency only if the required version has not been found by `pkg-config`;
//! - `always`: always build the dependency, ignoring any version which may be installed on the system;
//! - `never`: (default) never build the dependency, `system-deps` will fail if the required version is not found on the system.
//!
//! You can also use the `SYSTEM_DEPS_BUILD_INTERNAL` environment variable with the same values
//! defining the behavior for all the dependencies which don't have `SYSTEM_DEPS_$NAME_BUILD_INTERNAL` defined.

#![deny(missing_docs)]

#[cfg(test)]
#[macro_use]
extern crate lazy_static;

#[cfg(test)]
mod test;

use heck::{ShoutySnakeCase, SnakeCase};
use itertools::Itertools;
use std::collections::HashMap;
use std::env;
use std::fmt;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use strum::IntoEnumIterator;
use strum_macros::{EnumIter, EnumString};
use thiserror::Error;
use version_compare::VersionCompare;

mod metadata;
use metadata::MetaData;

/// system-deps errors
#[derive(Error, Debug)]
pub enum Error {
    /// pkg-config error
    #[error(transparent)]
    PkgConfig(#[from] pkg_config::Error),
    /// One of the `Config::add_build_internal` closures failed
    #[error("Failed to build {0}: {1}")]
    BuildInternalClosureError(String, #[source] BuildInternalClosureError),
    /// Failed to read `Cargo.toml`
    #[error("{0}")]
    FailToRead(String, #[source] std::io::Error),
    /// Raised when an error is detected in the metadata defined in `Cargo.toml`
    #[error("{0}")]
    InvalidMetadata(String),
    /// Raised when dependency defined manually using `SYSTEM_DEPS_$NAME_NO_PKG_CONFIG`
    /// did not define at least one lib using `SYSTEM_DEPS_$NAME_LIB` or
    /// `SYSTEM_DEPS_$NAME_LIB_FRAMEWORK`
    #[error("You should define at least one lib using {} or {}", EnvVariable::new_lib(.0).to_string(), EnvVariable::new_lib_framework(.0))]
    MissingLib(String),
    /// An environment variable in the form of `SYSTEM_DEPS_$NAME_BUILD_INTERNAL`
    /// contained an invalid value (allowed: `auto`, `always`, `never`)
    #[error("{0}")]
    BuildInternalInvalid(String),
    /// system-deps has been asked to internally build a lib, through
    /// `SYSTEM_DEPS_$NAME_BUILD_INTERNAL=always' or `SYSTEM_DEPS_$NAME_BUILD_INTERNAL=auto',
    /// but not closure has been defined using `Config::add_build_internal` to build
    /// this lib
    #[error("Missing build internal closure for {0} (version {1})")]
    BuildInternalNoClosure(String, String),
    /// The library which has been build internally does not match the
    /// required version defined in `Cargo.toml`
    #[error("Internally built {0} {1} but minimum required version is {2}")]
    BuildInternalWrongVersion(String, String, String),
    /// The `cfg()` expression used in `Cargo.toml` is currently not supported
    #[error("Unsupported cfg() expression: {0}")]
    UnsupportedCfg(String),
}

#[derive(Debug, Default)]
/// All the system dependencies retrieved by [Config::probe].
pub struct Dependencies {
    libs: HashMap<String, Library>,
}

impl Dependencies {
    /// Retrieve details about a system dependency.
    ///
    /// # Arguments
    ///
    /// * `name`: the name of the `toml` key defining the dependency in `Cargo.toml`
    pub fn get_by_name(&self, name: &str) -> Option<&Library> {
        self.libs.get(name)
    }

    /// An iterator visiting all system dependencies in arbitrary order.
    /// The first element of the tuple is the name of the `toml` key defining the
    /// dependency in `Cargo.toml`.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &Library)> {
        self.libs.iter().map(|(k, v)| (k.as_str(), v))
    }

    fn aggregate_str<F: Fn(&Library) -> &Vec<String>>(
        &self,
        getter: F,
    ) -> impl Iterator<Item = &str> {
        self.libs
            .values()
            .map(|l| getter(l))
            .flatten()
            .map(|s| s.as_str())
            .sorted()
            .dedup()
    }

    fn aggregate_path_buf<F: Fn(&Library) -> &Vec<PathBuf>>(
        &self,
        getter: F,
    ) -> impl Iterator<Item = &PathBuf> {
        self.libs
            .values()
            .map(|l| getter(l))
            .flatten()
            .sorted()
            .dedup()
    }

    /// An iterator returning each [Library::libs] of each library, removing duplicates.
    pub fn all_libs(&self) -> impl Iterator<Item = &str> {
        self.aggregate_str(|l| &l.libs)
    }

    /// An iterator returning each [Library::link_paths] of each library, removing duplicates.
    pub fn all_link_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.aggregate_path_buf(|l| &l.link_paths)
    }

    /// An iterator returning each [Library::frameworks] of each library, removing duplicates.
    pub fn all_frameworks(&self) -> impl Iterator<Item = &str> {
        self.aggregate_str(|l| &l.frameworks)
    }

    /// An iterator returning each [Library::framework_paths] of each library, removing duplicates.
    pub fn all_framework_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.aggregate_path_buf(|l| &l.framework_paths)
    }

    /// An iterator returning each [Library::include_paths] of each library, removing duplicates.
    pub fn all_include_paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.aggregate_path_buf(|l| &l.include_paths)
    }

    /// An iterator returning each [Library::defines] of each library, removing duplicates.
    pub fn all_defines(&self) -> impl Iterator<Item = (&str, &Option<String>)> {
        self.libs
            .values()
            .map(|l| l.defines.iter())
            .flatten()
            .map(|(k, v)| (k.as_str(), v))
            .sorted()
            .dedup()
    }

    fn add(&mut self, name: &str, lib: Library) {
        self.libs.insert(name.to_string(), lib);
    }

    fn override_from_flags(&mut self, env: &EnvVariables) {
        for (name, lib) in self.libs.iter_mut() {
            if let Some(value) = env.get(&EnvVariable::new_search_native(name)) {
                lib.link_paths = split_paths(&value);
            }
            if let Some(value) = env.get(&EnvVariable::new_search_framework(name)) {
                lib.framework_paths = split_paths(&value);
            }
            if let Some(value) = env.get(&EnvVariable::new_lib(name)) {
                lib.libs = split_string(&value);
            }
            if let Some(value) = env.get(&EnvVariable::new_lib_framework(name)) {
                lib.frameworks = split_string(&value);
            }
            if let Some(value) = env.get(&EnvVariable::new_include(name)) {
                lib.include_paths = split_paths(&value);
            }
        }
    }

    fn gen_flags(&self) -> Result<BuildFlags, Error> {
        let mut flags = BuildFlags::new();
        let mut include_paths = Vec::new();

        for (name, lib) in self.libs.iter() {
            include_paths.extend(lib.include_paths.clone());

            if lib.source == Source::EnvVariables
                && lib.libs.is_empty()
                && lib.frameworks.is_empty()
            {
                return Err(Error::MissingLib(name.clone()));
            }

            // lib.link_paths
            //     .iter()
            //     .for_each(|l| flags.add(BuildFlag::SearchNative(l.to_string_lossy().to_string())));
            // lib.framework_paths.iter().for_each(|f| {
            //     flags.add(BuildFlag::SearchFramework(f.to_string_lossy().to_string()))
            // });
            // lib.libs
            //     .iter()
            //     .for_each(|l| flags.add(BuildFlag::Lib(l.clone())));
            // lib.frameworks
            //     .iter()
            //     .for_each(|f| flags.add(BuildFlag::LibFramework(f.clone())));
        }

        // Export DEP_$CRATE_INCLUDE env variable with the headers paths,
        // see https://kornel.ski/rust-sys-crate#headers
        if !include_paths.is_empty() {
            if let Ok(paths) = std::env::join_paths(include_paths) {
                flags.add(BuildFlag::Include(paths.to_string_lossy().to_string()));
            }
        }

        // Export cargo:rerun-if-env-changed instructions for all env variables affecting system-deps behaviour
        flags.add(BuildFlag::RerunIfEnvChanged(
            EnvVariable::new_build_internal(None),
        ));

        for (name, _lib) in self.libs.iter() {
            for var in EnvVariable::iter() {
                let var = match var {
                    EnvVariable::Lib(_) => EnvVariable::new_lib(name),
                    EnvVariable::LibFramework(_) => EnvVariable::new_lib_framework(name),
                    EnvVariable::SearchNative(_) => EnvVariable::new_search_native(name),
                    EnvVariable::SearchFramework(_) => EnvVariable::new_search_framework(name),
                    EnvVariable::Include(_) => EnvVariable::new_include(name),
                    EnvVariable::NoPkgConfig(_) => EnvVariable::new_no_pkg_config(name),
                    EnvVariable::BuildInternal(_) => EnvVariable::new_build_internal(Some(name)),
                };
                flags.add(BuildFlag::RerunIfEnvChanged(var));
            }
        }

        Ok(flags)
    }
}

#[derive(Error, Debug)]
/// Error used in return value of `Config::add_build_internal` closures
pub enum BuildInternalClosureError {
    /// `pkg-config` error
    #[error(transparent)]
    PkgConfig(#[from] pkg_config::Error),
    /// General failure
    #[error("{0}")]
    Failed(String),
}

impl BuildInternalClosureError {
    /// Create a new `BuildInternalClosureError::Failed` representing a general
    /// failure.
    ///
    /// # Arguments
    ///
    /// * `details`: human-readable details about the failure
    pub fn failed(details: &str) -> Self {
        Self::Failed(details.to_string())
    }
}

// enums representing the environment variables user can define to tune system-deps
#[derive(Debug, PartialEq, EnumIter)]
enum EnvVariable {
    Lib(String),
    LibFramework(String),
    SearchNative(String),
    SearchFramework(String),
    Include(String),
    NoPkgConfig(String),
    BuildInternal(Option<String>),
}

impl EnvVariable {
    fn new_lib(lib: &str) -> Self {
        Self::Lib(lib.to_string())
    }

    fn new_lib_framework(lib: &str) -> Self {
        Self::LibFramework(lib.to_string())
    }

    fn new_search_native(lib: &str) -> Self {
        Self::SearchNative(lib.to_string())
    }

    fn new_search_framework(lib: &str) -> Self {
        Self::SearchFramework(lib.to_string())
    }

    fn new_include(lib: &str) -> Self {
        Self::Include(lib.to_string())
    }

    fn new_no_pkg_config(lib: &str) -> Self {
        Self::NoPkgConfig(lib.to_string())
    }

    fn new_build_internal(lib: Option<&str>) -> Self {
        Self::BuildInternal(lib.map(|l| l.to_string()))
    }

    fn suffix(&self) -> &'static str {
        match self {
            EnvVariable::Lib(_) => "LIB",
            EnvVariable::LibFramework(_) => "LIB_FRAMEWORK",
            EnvVariable::SearchNative(_) => "SEARCH_NATIVE",
            EnvVariable::SearchFramework(_) => "SEARCH_FRAMEWORK",
            EnvVariable::Include(_) => "INCLUDE",
            EnvVariable::NoPkgConfig(_) => "NO_PKG_CONFIG",
            EnvVariable::BuildInternal(_) => "BUILD_INTERNAL",
        }
    }
}

impl fmt::Display for EnvVariable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let suffix = match self {
            EnvVariable::Lib(lib)
            | EnvVariable::LibFramework(lib)
            | EnvVariable::SearchNative(lib)
            | EnvVariable::SearchFramework(lib)
            | EnvVariable::Include(lib)
            | EnvVariable::NoPkgConfig(lib)
            | EnvVariable::BuildInternal(Some(lib)) => {
                format!("{}_{}", lib.to_shouty_snake_case(), self.suffix())
            }
            EnvVariable::BuildInternal(None) => self.suffix().to_string(),
        };
        write!(f, "SYSTEM_DEPS_{}", suffix)
    }
}

type FnBuildInternal =
    dyn FnOnce(&str, &str) -> std::result::Result<Library, BuildInternalClosureError>;

/// Structure used to configure `metadata` before starting to probe for dependencies
pub struct Config {
    env: EnvVariables,
    build_internals: HashMap<String, Box<FnBuildInternal>>,
}

impl Default for Config {
    fn default() -> Self {
        Self::new_with_env(EnvVariables::Environnement)
    }
}

impl Config {
    /// Create a new set of configuration
    pub fn new() -> Self {
        Self::default()
    }

    fn new_with_env(env: EnvVariables) -> Self {
        Self {
            env,
            build_internals: HashMap::new(),
        }
    }

    /// Probe all libraries configured in the Cargo.toml
    /// `[package.metadata.system-deps]` section.
    ///
    /// The returned hash is using the the `toml` key defining the dependency as key.
    pub fn probe(self) -> Result<Dependencies, Error> {
        let libraries = self.probe_full()?;
        let flags = libraries.gen_flags()?;

        // Output cargo flags
        println!("{}", flags);

        for (name, _) in libraries.iter() {
            println!("cargo:rustc-cfg=system_deps_have_{}", name.to_snake_case());
        }

        Ok(libraries)
    }

    /// Add hook so system-deps can internally build library `name` if requested by user.
    ///
    /// It will only be triggered if the environment variable
    /// `SYSTEM_DEPS_$NAME_BUILD_INTERNAL` is defined with either `always` or
    /// `auto` as value. In the latter case, `func` is called only if the requested
    /// version of the library was not found on the system.
    ///
    /// # Arguments
    /// * `name`: the name of the library, as defined in `Cargo.toml`
    /// * `func`: closure called when internally building the library.
    /// It receives as argument the library name and the minimum version required.
    pub fn add_build_internal<F>(self, name: &str, func: F) -> Self
    where
        F: 'static + FnOnce(&str, &str) -> std::result::Result<Library, BuildInternalClosureError>,
    {
        let mut build_internals = self.build_internals;
        build_internals.insert(name.to_string(), Box::new(func));

        Self {
            env: self.env,
            build_internals,
        }
    }

    fn probe_full(mut self) -> Result<Dependencies, Error> {
        let mut libraries = self.probe_pkg_config()?;
        libraries.override_from_flags(&self.env);

        Ok(libraries)
    }

    fn probe_pkg_config(&mut self) -> Result<Dependencies, Error> {
        let dir = self
            .env
            .get("CARGO_MANIFEST_DIR")
            .ok_or_else(|| Error::InvalidMetadata("$CARGO_MANIFEST_DIR not set".into()))?;
        let mut path = PathBuf::from(dir);
        path.push("Cargo.toml");

        let metadata = MetaData::from_file(&path)?;

        let mut libraries = Dependencies::default();

        for dep in metadata.deps.iter() {
            if let Some(cfg) = &dep.cfg {
                // Check if `cfg()` expression matches the target settings
                if !self.check_cfg(cfg)? {
                    continue;
                }
            }

            let mut enabled_feature_overrides = Vec::new();

            for o in dep.version_overrides.iter() {
                if self.has_feature(&o.key) {
                    enabled_feature_overrides.push(o);
                }
            }

            if let Some(feature) = dep.feature.as_ref() {
                if !self.has_feature(&feature) {
                    continue;
                }
            }

            let (version, lib_name, optional) = {
                // Pick the highest feature enabled version
                if !enabled_feature_overrides.is_empty() {
                    enabled_feature_overrides.sort_by(|a, b| {
                        VersionCompare::compare(&a.version, &b.version)
                            .expect("failed to compare versions")
                            .ord()
                            .expect("invalid version")
                    });
                    let highest = enabled_feature_overrides.into_iter().last().unwrap();
                    (
                        Some(&highest.version),
                        highest.name.clone().unwrap_or_else(|| dep.lib_name()),
                        highest.optional.unwrap_or(dep.optional),
                    )
                } else {
                    (dep.version.as_ref(), dep.lib_name(), dep.optional)
                }
            };

            let version = version.ok_or_else(|| {
                Error::InvalidMetadata(format!("No version defined for {}", dep.key))
            })?;

            let name = &dep.key;
            let build_internal = self.get_build_internal_status(name)?;

            let library = if self.env.contains(&EnvVariable::new_no_pkg_config(name)) {
                Library::from_env_variables(name)
            } else if build_internal == BuildInternal::Always {
                self.call_build_internal(&lib_name, &version)?
            } else {
                match pkg_config::Config::new()
                    .atleast_version(&version)
                    .print_system_libs(false)
                    .cargo_metadata(true)
                    .statik(true)
                    .probe(&lib_name)
                {
                    Ok(lib) => Library::from_pkg_config(&lib_name, lib),
                    Err(e) => {
                        if build_internal == BuildInternal::Auto {
                            // Try building the lib internally as a fallback
                            self.call_build_internal(name, &version)?
                        } else if optional {
                            // If the dep is optional just skip it
                            continue;
                        } else {
                            return Err(e.into());
                        }
                    }
                }
            };

            libraries.add(name, library);
        }
        Ok(libraries)
    }

    fn get_build_internal_env_var(&self, var: EnvVariable) -> Result<Option<BuildInternal>, Error> {
        match self.env.get(&var).as_deref() {
            Some(s) => {
                let b = BuildInternal::from_str(s).map_err(|_| {
                    Error::BuildInternalInvalid(format!(
                        "Invalid value in {}: {} (allowed: 'auto', 'always', 'never')",
                        var, s
                    ))
                })?;
                Ok(Some(b))
            }
            None => Ok(None),
        }
    }

    fn get_build_internal_status(&self, name: &str) -> Result<BuildInternal, Error> {
        match self.get_build_internal_env_var(EnvVariable::new_build_internal(Some(name)))? {
            Some(b) => Ok(b),
            None => Ok(self
                .get_build_internal_env_var(EnvVariable::new_build_internal(None))?
                .unwrap_or_default()),
        }
    }

    fn call_build_internal(&mut self, name: &str, version: &str) -> Result<Library, Error> {
        let lib = match self.build_internals.remove(name) {
            Some(f) => {
                f(name, version).map_err(|e| Error::BuildInternalClosureError(name.into(), e))?
            }
            None => return Err(Error::BuildInternalNoClosure(name.into(), version.into())),
        };

        // Check that the lib built internally matches the required version
        match VersionCompare::compare(&lib.version, version) {
            Ok(version_compare::CompOp::Lt) => Err(Error::BuildInternalWrongVersion(
                name.into(),
                lib.version.clone(),
                version.into(),
            )),
            _ => Ok(lib),
        }
    }

    fn has_feature(&self, feature: &str) -> bool {
        let var: &str = &format!("CARGO_FEATURE_{}", feature.to_uppercase().replace('-', "_"));
        self.env.contains(var)
    }

    fn check_cfg(&self, cfg: &cfg_expr::Expression) -> Result<bool, Error> {
        use cfg_expr::{targets::get_builtin_target_by_triple, Predicate};

        let target = self
            .env
            .get("TARGET")
            .expect("no TARGET env variable defined");
        let target = get_builtin_target_by_triple(&target)
            .unwrap_or_else(|| panic!("Invalid TARGET: {}", target));

        let res = cfg.eval(|pred| match pred {
            Predicate::Target(tp) => Some(tp.matches(target)),
            _ => None,
        });

        res.ok_or_else(|| Error::UnsupportedCfg(cfg.original().to_string()))
    }
}

#[derive(Debug, PartialEq)]
/// From where the library settings have been retrieved
pub enum Source {
    /// Settings have been retrieved from `pkg-config`
    PkgConfig,
    /// Settings have been defined using user defined environment variables
    EnvVariables,
}

#[derive(Debug)]
/// A system dependency
pub struct Library {
    /// Name of the library
    pub name: String,
    /// From where the library settings have been retrieved
    pub source: Source,
    /// libraries the linker should link on
    pub libs: Vec<String>,
    /// directories where the compiler should look for libraries
    pub link_paths: Vec<PathBuf>,
    /// frameworks the linker should link on
    pub frameworks: Vec<String>,
    /// directories where the compiler should look for frameworks
    pub framework_paths: Vec<PathBuf>,
    /// directories where the compiler should look for header files
    pub include_paths: Vec<PathBuf>,
    /// macros that should be defined by the compiler
    pub defines: HashMap<String, Option<String>>,
    /// library version
    pub version: String,
}

impl Library {
    fn from_pkg_config(name: &str, l: pkg_config::Library) -> Self {
        Self {
            name: name.to_string(),
            source: Source::PkgConfig,
            libs: l.libs,
            link_paths: l.link_paths,
            include_paths: l.include_paths,
            frameworks: l.frameworks,
            framework_paths: l.framework_paths,
            defines: l.defines,
            version: l.version,
        }
    }

    fn from_env_variables(name: &str) -> Self {
        Self {
            name: name.to_string(),
            source: Source::EnvVariables,
            libs: Vec::new(),
            link_paths: Vec::new(),
            include_paths: Vec::new(),
            frameworks: Vec::new(),
            framework_paths: Vec::new(),
            defines: HashMap::new(),
            version: String::new(),
        }
    }

    /// Create a `Library` by probing `pkg-config` on an internal directory.
    /// This helper is meant to be used by `Config::add_build_internal` closures
    /// after having built the lib to return the library information to system-deps.
    ///
    /// # Arguments
    ///
    /// * `pkg_config_dir`: the directory where the library `.pc` file is located
    /// * `lib`: the name of the library to look for
    /// * `version`: the minimum version of `lib` required
    ///
    /// # Examples
    ///
    /// ```
    /// let mut config = system_deps::Config::new();
    /// config.add_build_internal("mylib", |lib, version| {
    ///   // Actually build the library here
    ///   system_deps::Library::from_internal_pkg_config("build-dir",
    ///       lib, version)
    /// });
    /// ```
    pub fn from_internal_pkg_config<P>(
        pkg_config_dir: P,
        lib: &str,
        version: &str,
    ) -> Result<Self, BuildInternalClosureError>
    where
        P: AsRef<Path>,
    {
        // save current PKG_CONFIG_PATH so we can restore it
        let old = env::var("PKG_CONFIG_PATH");

        match old {
            Ok(ref s) => {
                let mut paths = env::split_paths(s).collect::<Vec<_>>();
                paths.push(PathBuf::from(pkg_config_dir.as_ref()));
                let paths = env::join_paths(paths).unwrap();
                env::set_var("PKG_CONFIG_PATH", paths)
            }
            Err(_) => env::set_var("PKG_CONFIG_PATH", pkg_config_dir.as_ref()),
        }

        let pkg_lib = pkg_config::Config::new()
            .atleast_version(&version)
            .print_system_libs(false)
            .cargo_metadata(true)
            .statik(true)
            .probe(lib);

        env::set_var("PKG_CONFIG_PATH", &old.unwrap_or_else(|_| "".into()));

        match pkg_lib {
            Ok(pkg_lib) => Ok(Self::from_pkg_config(&lib, pkg_lib)),
            Err(e) => Err(e.into()),
        }
    }
}

#[derive(Debug)]
enum EnvVariables {
    Environnement,
    #[cfg(test)]
    Mock(HashMap<&'static str, String>),
}

trait EnvVariablesExt<T> {
    fn contains(&self, var: T) -> bool {
        self.get(var).is_some()
    }
    fn get(&self, var: T) -> Option<String>;
}

impl EnvVariablesExt<&str> for EnvVariables {
    fn get(&self, var: &str) -> Option<String> {
        match self {
            EnvVariables::Environnement => env::var(var).ok(),
            #[cfg(test)]
            EnvVariables::Mock(vars) => vars.get(var).cloned(),
        }
    }
}

impl EnvVariablesExt<&EnvVariable> for EnvVariables {
    fn get(&self, var: &EnvVariable) -> Option<String> {
        let s = var.to_string();
        let var: &str = s.as_ref();
        self.get(var)
    }
}

// TODO: add support for "rustc-link-lib=static=" ?
#[derive(Debug, PartialEq)]
enum BuildFlag {
    Include(String),
    SearchNative(String),
    SearchFramework(String),
    Lib(String),
    LibFramework(String),
    RerunIfEnvChanged(EnvVariable),
}

impl fmt::Display for BuildFlag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildFlag::Include(paths) => write!(f, "include={}", paths),
            BuildFlag::SearchNative(lib) => write!(f, "rustc-link-search=native={}", lib),
            BuildFlag::SearchFramework(lib) => write!(f, "rustc-link-search=framework={}", lib),
            BuildFlag::Lib(lib) => write!(f, "rustc-link-lib={}", lib),
            BuildFlag::LibFramework(lib) => write!(f, "rustc-link-lib=framework={}", lib),
            BuildFlag::RerunIfEnvChanged(env) => write!(f, "rerun-if-env-changed={}", env),
        }
    }
}

#[derive(Debug, PartialEq)]
struct BuildFlags(Vec<BuildFlag>);

impl BuildFlags {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn add(&mut self, flag: BuildFlag) {
        self.0.push(flag);
    }
}

impl fmt::Display for BuildFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for flag in self.0.iter() {
            writeln!(f, "cargo:{}", flag)?;
        }
        Ok(())
    }
}

fn split_paths(value: &str) -> Vec<PathBuf> {
    if !value.is_empty() {
        let paths = env::split_paths(&value);
        paths.map(|p| Path::new(&p).into()).collect()
    } else {
        Vec::new()
    }
}

fn split_string(value: &str) -> Vec<String> {
    if !value.is_empty() {
        value.split(' ').map(|s| s.to_string()).collect()
    } else {
        Vec::new()
    }
}

#[derive(Debug, PartialEq, EnumString)]
#[strum(serialize_all = "snake_case")]
enum BuildInternal {
    Auto,
    Always,
    Never,
}

impl Default for BuildInternal {
    fn default() -> Self {
        BuildInternal::Never
    }
}
