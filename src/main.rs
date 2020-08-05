#[macro_use] extern crate lazy_static;
use std::collections::HashMap;
use regex::Regex;
use ureq;
use clap::Clap;

/// Pkg: debian package struct that holds basic information about a package
#[derive(Debug, Clone)]
pub struct Pkg {
    /// package name
    name: String,
    /// packge version
    version: Option<String>,
    /// packge source
    source: Option<String>,
    /// package homepage
    homepage: Option<String>,
    /// git url
    git_url: Option<String>,
    /// all dependencies
    deps: Vec<String>,
    /// internal dependencies
    int_deps: Vec<String>,
}

impl Pkg {
    pub fn add_dependencies(&mut self, deps_string: String) {
        lazy_static!{
            static ref RE: Regex = Regex::new(r"^(.*?)(:.*)?( \(.*\))?$").unwrap();
        }

        let mut deps = vec!();
        for dep in deps_string.split(", ") {
            let m = RE.captures(dep).unwrap()[1].to_string();
            if ! deps.contains(&m) {
                deps.push(m);
            }
        }
        self.deps = deps;
    }

    pub fn new(pkg_string: &str) -> Option<Pkg> {
        if pkg_string == "" {
            return None
        }
        lazy_static!{
            static ref RE: Regex = Regex::new(r"([A-Za-z0-9]+): (.*)$").unwrap();
        }
        let mut processed_lines: Vec<String> = vec!();
        for line in pkg_string.split("\n"){
            match line.chars().next() {
                Some(' ') => {
                    // extend the last one line
                    let latest = processed_lines.pop().unwrap();
                    processed_lines.push(format!("{} {}", latest, line));
                },
                Some(_) => {
                    processed_lines.push(line.to_owned())
                },
                None => {}
            }
        }

        let mut pkg = Pkg {
            name: "".to_owned(),
            version: None,
            source: None,
            homepage: None,
            git_url: None,
            deps: vec!(),
            int_deps: vec!()
        };

        for line in processed_lines {
            let caps = RE.captures(&line).unwrap();
            let key = caps.get(1).map_or("", |m| m.as_str());
            let value = caps.get(2).map_or("", |m| m.as_str()).to_string();
            match key {
                "Package" => {
                    pkg.name = value;
                }
                "Depends" => {
                    // dependencies
                    pkg.add_dependencies(value);
                }
                "Homepage" => {
                    pkg.homepage = Some(value);
                  
                }
                "Version" => {
                    pkg.version = Some(value);
                }
                "Source" => {
                    pkg.source = Some(value);
                }
                _ => {}
            }
        }

        Some(pkg)
    }
}

/// get all packages of all versions
fn get_all_packages(os: &String, distro: &String) -> Vec<Pkg> {
    // get Package content
    let url = format!("https://pkg.caida.org/os/{}/dists/{}/main/binary-amd64/Packages", os, distro);
    let resp = ureq::get(url.as_str()).call();
    match resp.into_string() {
        Ok(pkg_str) => {
            if pkg_str.contains("could not be found") {
                panic!(format!("Unable to retrive Packages from {}", url))
            }
            pkg_str.split("\n\n").collect::<Vec<&str>>()
                .into_iter()
                .filter_map(|l| Pkg::new(l))
                .collect::<Vec<Pkg>>()
        },
        Err(_) => {
            panic!(format!("Unable to retrive Packages from {}", url))
        }
    }
}

fn process_packages(pkgs: Vec<Pkg>) -> Vec<Pkg> {
    let mut pkg_map:HashMap<String, Pkg> = HashMap::new();
    for pkg in pkgs {
        if !pkg_map.contains_key(&pkg.name){
            pkg_map.insert(pkg.name.to_string(), pkg.to_owned());
        }
    }
    let pkg_names = pkg_map.keys().cloned().into_iter().collect::<Vec<String>>();
    for pkg in pkg_map.values_mut() {
        pkg.int_deps = pkg.deps.clone()
           .into_iter()
           .filter(|x| pkg_names.contains(x))
           .collect()
    }

    // order by internal depedencies
    let mut left_over: Vec<Pkg> = pkg_map.values().cloned().into_iter().collect();
    let target_len = left_over.len();
    let mut ordered_pkgs:Vec<String> = vec!();

    while ordered_pkgs.len() < target_len {
        for pkg in &left_over{
            let left_deps:Vec<&String> = pkg.int_deps
               .iter()
               .filter(|x| !ordered_pkgs.contains(x)).collect();
            if left_deps.len()==0 {
                // no other internal dependencies, add to ordered_pkgs
                ordered_pkgs.push(pkg.name.clone());
            }
        }
        left_over = left_over
            .into_iter()
            .filter(|x| !ordered_pkgs.contains(&x.name))
            .collect();
    }

    ordered_pkgs.into_iter().filter_map(|x| pkg_map.get(&x)).cloned().collect()
}

fn extract_pkg_or_sources(pkgs: Vec<Pkg>) -> Vec<String> {
    let mut sources:Vec<String> = vec!();

    for pkg in pkgs {
        match pkg.source {
            Some(source) =>{
                if !sources.contains(&source) {
                    sources.push(source)
                }
            }
            None => {
                if !sources.contains(&pkg.name) {
                    sources.push(pkg.name)
                }
            }
        }
    }
    sources
}

/// This program extracts the Packages file from the specified OS/Distribution
/// on the CAIDA Debian package repository, and print out the list of package
/// sources in a order that follows internal dependency.
#[derive(Clap, Debug)]
#[clap()]
struct Opts {
    #[clap(short, long)]
    os: String,
    #[clap(short, long)]
    distro: String
}

fn main() {
    let opts: Opts = Opts::parse();
    let pkgs = process_packages(get_all_packages(&opts.os, &opts.distro));
    for source in extract_pkg_or_sources(pkgs){
        println!("{}", source)
    }
}
