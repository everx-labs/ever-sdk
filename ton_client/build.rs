/*
 * Copyright 2018-2021 TON Labs LTD.
 *
 * Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
 * this file except in compliance with the License.
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific TON DEV software governing permissions and
 * limitations under the License.
 *
 */

use serde_json::Value;
use std::path::PathBuf;
use std::process::Command;

extern crate serde_json;
#[macro_use]
extern crate serde_derive;

#[derive(Serialize, Debug)]
struct Dep {
    name: String,
    version: String,
    git_commit: String,
}

#[derive(Serialize, Debug)]
struct BuildInfo {
    build_number: u32,
    dependencies: Vec<Dep>,
}

fn exec(cmd: &str, args: &[&str]) -> Result<String, String> {
    let out = Command::new(cmd).args(args).output().unwrap();
    if out.status.success() {
        Ok(String::from_utf8(out.stdout).unwrap())
    } else {
        Err(String::from_utf8(out.stderr).unwrap())
    }
}

fn root() -> PathBuf {
    PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
}

impl BuildInfo {
    fn load() -> Result<Self, String> {
        let manifest_path = root().join("Cargo.toml");
        let meta_out = exec(
            "cargo",
            &[
                "metadata",
                "--locked",
                "--format-version",
                "1",
                "--manifest-path",
                manifest_path.to_str().unwrap(),
            ],
        )?;
        let meta = serde_json::from_str::<Value>(&meta_out).unwrap();
        let git_commit = exec("git", &["rev-parse", "HEAD"])?.trim().to_string();
        let mut build_number = std::env::var("TON_BUILD_NUMBER").unwrap_or("".into());
        let dependencies: Vec<Dep> = meta["packages"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|x| {
                let source = x["source"].as_str().unwrap_or("");
                source.is_empty() || source.contains("tonlabs")
            })
            .map(|x| {
                let name = x["name"].as_str().unwrap().to_string();
                let version = x["version"].as_str().unwrap().to_string();
                if build_number.is_empty() && name == "ton_client" {
                    build_number = version.split(".").map(|x| format!("{:0>3}", x)).collect();
                }
                let source = x["source"].as_str().unwrap_or("");
                let git_commit = if !source.is_empty() {
                    source.split("#").last().unwrap_or("").to_string()
                } else {
                    git_commit.clone()
                };
                Dep {
                    name,
                    version,
                    git_commit,
                }
            })
            .collect();
        Ok(Self {
            build_number: u32::from_str_radix(&build_number, 10)
                .expect(&format!("Invalid build number [{}]", build_number)),
            dependencies,
        })
    }
}

fn main() -> Result<(), String> {
    /* FIXME: failed with `the lock file Cargo.lock needs to be updated but
        --locked was passed to prevent this. If you want to try to generate
        the lock file without accessing the network, use the --offline flag.`
    */
    if false {
        let build_info = BuildInfo::load()?;
        let build_info_json = serde_json::to_string_pretty(&build_info).unwrap();
        std::fs::write(
            root().join("src").join("build_info.json").to_str().unwrap(),
            build_info_json,
        )
        .map_err(|err| format!("{}", err))?;
    }
    Ok(())
}
