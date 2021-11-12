#![allow(unused_imports, dead_code, unused_variables, unused_mut, unused_must_use)]
mod grafana;
use std::io::Write;

#[macro_use]
extern crate log;
extern crate env_logger;

extern crate clap_v3;
use clap_v3::{App, Arg};

fn main() -> Result<(), anyhow::Error> {
  let app = App::new("").version(env!("CARGO_PKG_VERSION")).author(env!("CARGO_PKG_AUTHORS")).about(env!("CARGO_PKG_DESCRIPTION")).arg(Arg::with_name("host").short('h').long("host").env("HOST").required(true).help("http://host:port")).arg(Arg::with_name("auth_usr").short('u').long("auth_usr").env("AUTH_USR").required(true)).arg(Arg::with_name("auth_pwd").short('p').long("auth_pwd").env("AUTH_PWD").requires("auth_usr").required(true)).arg(Arg::with_name("export_path").short('e').long("export_path").env("EXPORT_PATH").required(false).default_value("export").help("Export path")).arg(Arg::with_name("v").short('v').multiple(true).takes_value(false).required(false).help("Log verbosity (-v, -vv, -vvv...)")).get_matches();

  match app.occurrences_of("v") {
    0 => std::env::set_var("RUST_LOG", "error"),
    1 => std::env::set_var("RUST_LOG", "warn"),
    2 => std::env::set_var("RUST_LOG", "info"),
    3 => std::env::set_var("RUST_LOG", "debug"),
    4 | _ => std::env::set_var("RUST_LOG", "trace"),
  }
  env_logger::Builder::from_default_env().format(|buf, record| writeln!(buf, "{} {} {}:{} [{}] - {}", chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"), record.module_path().unwrap_or("unknown"), record.file().unwrap_or("unknown"), record.line().unwrap_or(0), record.level(), record.args())).init();

  let mut gra = grafana::client::ClientInfo::new(app.value_of("export_path"), app.value_of("host"), app.value_of("auth_usr"), app.value_of("auth_pwd"));

  if let Ok(orgs) = gra.get_orgs() {
    for (org_id, org_name) in orgs {
      if gra.set_org(org_id.clone()).is_ok() {
        if let Ok(ds) = gra.search_datasources() {
          for (ds_id, _) in ds {
            gra.save_datasources(org_name.clone(), ds_id)?;
          }
        }
        if let Ok(dashs) = gra.search_dashboards() {
          for (dash_id, _) in dashs {
            gra.save_dashboards(org_name.clone(), dash_id)?;
          }
        }
      }
    }
  }

  Ok(())
}
