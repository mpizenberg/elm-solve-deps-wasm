// SPDX-License-Identifier: MPL-2.0

//! WebAssembly module to solve dependencies in the elm ecosystem.
#![warn(clippy::pedantic)]

use std::collections::HashMap;
use std::str::FromStr;

use anyhow::Context;
use pubgrub::error::PubGrubError;
use pubgrub::report::{DefaultStringReporter, Reporter};
use pubgrub::version::SemanticVersion as SemVer;
use wee_alloc::WeeAlloc;

use elm_solve_deps::constraint::Constraint;
use elm_solve_deps::project_config::{Pkg, ProjectConfig};
use elm_solve_deps::solver::solve_deps_with;

use wasm_bindgen::prelude::*;

mod utils;

// Use `wee_alloc` as the global allocator.
#[global_allocator]
static ALLOC: WeeAlloc = WeeAlloc::INIT;

/// Initialize the panic hook for more meaningful errors in case of panics,
/// and also initialize the logger for the wasm code.
///
/// # Panics
///
/// Will panic if the logger cannot be initialized.
#[wasm_bindgen]
pub fn init() {
    utils::set_panic_hook();
    utils::WasmLogger::init().unwrap();
    utils::WasmLogger::setup(utils::verbosity_filter(2)); // INFO
}

#[wasm_bindgen]
/// Some types for the TS bindings.
extern "C" {
    #[wasm_bindgen(typescript_type = "Record<string, string>")]
    pub type AdditionalConstraintsStr;

    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "(pkg: string) => string[]")]
    pub type JsListAvailableVersions;

    #[wasm_bindgen(extends = js_sys::Function, typescript_type = "(pkg: string, version: string) => string")]
    pub type JsFetchElmJson;
}

/// Solve dependencies for the provided `elm.json`.
///
/// Include also test dependencies if `use_test` is `true`.
/// It is possible to add additional constraints.
/// The caller is responsible to provide implementations to be able to fetch the `elm.json` of
/// dependencies, as well as to list existing versions (in preferred order) for a given package.
///
/// # Errors
///
/// If there is a PubGrub error, it will be reported.
///
/// # Panics
///
/// If the `elm.json` cannot be decoded, it will panic.
///
#[wasm_bindgen]
pub fn solve_deps(
    project_elm_json_str: &str,
    use_test: bool,
    additional_constraints_str: AdditionalConstraintsStr,
    js_fetch_elm_json: &JsFetchElmJson,
    js_list_available_versions: &JsListAvailableVersions,
) -> Result<String, JsValue> {
    // Load the elm.json of the package given as argument or of the current folder.
    let project_elm_json: ProjectConfig = serde_json::from_str(project_elm_json_str)
        .context("Failed to decode the elm.json")
        .map_err(utils::report_error)?;

    // Parse additional constraints.
    let additional_constraints: HashMap<String, String> =
        serde_wasm_bindgen::from_value(additional_constraints_str.into())?;
    let additional_constraints: Vec<(Pkg, Constraint)> = additional_constraints
        .into_iter()
        .map(|(pkg, constraint)| {
            Ok((
                Pkg::from_str(&pkg).map_err(utils::report_error)?,
                Constraint::from_str(&constraint).map_err(utils::report_error)?,
            ))
        })
        .collect::<Result<_, JsValue>>()?;

    let fetch_elm_json = |pkg: &Pkg, version: SemVer| {
        let js_pkg = JsValue::from_str(&pkg.to_string());
        let js_version = JsValue::from_str(&version.to_string());
        match js_fetch_elm_json.call2(&JsValue::NULL, &js_pkg, &js_version) {
            Ok(js_config) => {
                let str_config = js_config.as_string().context("Not a string?")?;
                Ok(serde_json::from_str(&str_config)?)
            }
            Err(js_err) => {
                let str_js_err =
                    js_sys::JSON::stringify(&js_err).unwrap_or_else(|_| js_sys::JsString::from(""));
                Err(format!(
                    "An error occurred in the JS function call `fetch_elm_json({pkg}, {version})`.\n\n{str_js_err}"
                )
                .into())
            }
        }
    };

    let list_available_versions = |pkg: &Pkg| match js_list_available_versions
        .call1(&JsValue::NULL, &JsValue::from_str(&pkg.to_string()))
    {
        Ok(js_versions) => {
            let versions: Vec<String> = serde_wasm_bindgen::from_value(js_versions)?;
            Ok(versions
                .into_iter()
                .filter_map(|v| SemVer::from_str(&v).ok()))
        }
        Err(js_err) => {
            let str_js_err =
                js_sys::JSON::stringify(&js_err).unwrap_or_else(|_| js_sys::JsString::from(""));
            Err(format!(
                "An error occurred in the JS function call `list_available_versions({pkg})`.\n\n{str_js_err}"
            )
            .into())
        }
    };

    match solve_deps_with(
        &project_elm_json,
        use_test,
        &additional_constraints,
        fetch_elm_json,
        list_available_versions,
    ) {
        Ok(solution) => {
            let solution_json = serde_json::to_string(&solution).unwrap();
            Ok(solution_json)
        }
        Err(err) => Err(utils::report_error(handle_pubgrub_error(err))),
    }
}

// Helper functions ######################################################################

fn handle_pubgrub_error(err: PubGrubError<Pkg, SemVer>) -> anyhow::Error {
    match err {
        PubGrubError::NoSolution(tree) => {
            anyhow::anyhow!(DefaultStringReporter::report(&tree))
        }
        PubGrubError::ErrorRetrievingDependencies {
            package,
            version,
            source,
        } => anyhow::anyhow!(
            "An error occured while trying to retrieve dependencies of {}@{}:\n\n{}",
            package,
            version,
            source
        ),
        PubGrubError::DependencyOnTheEmptySet {
            package,
            version,
            dependent,
        } => anyhow::anyhow!(
            "{}@{} has an impossible dependency on {}",
            package,
            version,
            dependent
        ),
        PubGrubError::SelfDependency { package, version } => {
            anyhow::anyhow!("{}@{} somehow depends on itself", package, version)
        }
        PubGrubError::ErrorChoosingPackageVersion(err) => anyhow::anyhow!(
            "There was an error while picking packages for dependency resolution:\n\n{}",
            err
        ),
        PubGrubError::ErrorInShouldCancel(err) => {
            anyhow::anyhow!("Dependency resolution was cancelled.\n\n{}", err)
        }
        PubGrubError::Failure(err) => anyhow::anyhow!(
            "An unrecoverable error happened while solving dependencies:\n\n{}",
            err
        ),
    }
}
