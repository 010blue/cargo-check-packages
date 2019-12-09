extern crate reqwest;

use regex::Regex;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::path::Path;
use toml;

// get package's max_version
fn get_max_version(package_name: &str) -> reqwest::Result<(String)> {
    let uri = "https://crates.io/api/v1/crates/".to_owned() + package_name;

    let res = reqwest::get(&uri)?
        .text()?;


    let mut data: serde_json::Value = serde_json::Value::Null;
    match serde_json::from_str(&res) {
        Ok(v) => data = v,
        Err(e) => println!("{}", e),
    }

    let max_version = &data["crate"]["max_version"].to_string();
    Ok(max_version.to_string())
}

fn version_to_arr(version: &str) -> Vec<i32> {
    let re = Regex::new("[^.\\d]+").unwrap();
    let version_new = re.replace(version, "");
    let vs = version_new
        .split(".")
        .map(|a| if a == "" { 0 } else { a.parse::<i32>().unwrap() })
        .collect();
    vs
}

// compare versions
// if version < max_version => true
fn is_low_version(version: &str, max_version: &str) -> bool {
    let vs = version_to_arr(version);
    let mvs = version_to_arr(max_version);
    
    let mut re = false;

    match vs.len() {
        1 => if mvs.len() > 0 && vs[0] < mvs[0] { re = true },
        2 => if mvs.len() > 1 && (vs[0] < mvs[0] || vs[1] < mvs[1]) { re = true },
        3 => if mvs.len() > 2 && (vs[0] < mvs[0] || vs[1] < mvs[1] || vs[2] < mvs[2]) { re = true },
        _ => re = true 
    }

    re
}

fn main() {
    // get_package_max_version("toml");
    // read packages from Cargo.lock
    let file_name = "./Cargo.lock";
    let path = Path::new(file_name);

    let mut file = match File::open(&path) {
        Err(why) => panic!("{}, try to run `cargo generate-lockfile first`", why.description()),
        Ok(file) => file,
    };

    let mut file_str = String::new();
    match file.read_to_string(&mut file_str) {
        Err(why) => panic!("{}", why.description()),
        Ok(_) => (),
    }

    let data = file_str.parse::<toml::Value>().unwrap();
    let packages = data["package"].as_array();

    let mut low_packages_num = 0;
    for pac in packages {
        println!("{} packages found, start checking...", pac.len());
        for p in pac {
            let name = str::replace(&p["name"].to_string(), "\"", "");
            let version = str::replace(&p["version"].to_string(), "\"", "");
            // get max version
            let mut max_version = "".to_string();
            let max_version_res = get_max_version(&name);
            match max_version_res {
                Ok(v) => max_version = str::replace(&v, "\"", ""),
                Err(e) => println!("{}", e),
            }

            if is_low_version(&version, &max_version) {
                low_packages_num = low_packages_num + 1;
                println!("{} 's version {} is lower than max_version {}", name, version, max_version);
            }
        }
    }

    println!("{} low packages found, done.", low_packages_num);
}
