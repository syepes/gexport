use anyhow::{anyhow, Result};
use reqwest::StatusCode;
use serde_json::Value;
use std::{collections::HashMap, fs, fs::File, io::BufWriter, path::Path, time::Duration};

#[derive(Debug, Default)]
pub struct ClientInfo<'a> {
  pub cfg_path: Option<&'a str>,
  pub ip:       Option<&'a str>,
  pub auth_usr: Option<&'a str>,
  pub auth_pwd: Option<&'a str>,
}

impl<'a> ClientInfo<'a> {
  pub fn new(cfg_path: Option<&'a str>, ip: Option<&'a str>, auth_usr: Option<&'a str>, auth_pwd: Option<&'a str>) -> ClientInfo<'a> {
    ClientInfo { cfg_path,
                 ip,
                 auth_usr,
                 auth_pwd }
  }

  pub fn get_orgs(&mut self) -> Result<HashMap<String, String>, anyhow::Error> {
    trace!("get_orgs");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(5)).connection_verbose(true).build() {
      if !self.auth_usr.unwrap().is_empty() && !self.auth_pwd.unwrap().is_empty() {
        let req_url = format!("{ip}/api/orgs", ip = self.ip.unwrap());
        trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

        let req = c.get(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
        match req.send() {
          Ok(r) => {
            trace!("resp:{:#?}", r);
            match r.status() {
              StatusCode::OK => {
                match r.json::<serde_json::Value>() {
                  Ok(t) => {
                    trace!("resp.json: {:#?}", t);
                    let mut orgs: HashMap<String, String> = HashMap::new();
                    for v in t.as_array().into_iter() {
                      for o in v.iter() {
                        orgs.entry(sanitize_names(o.get("id").unwrap().to_string())).or_insert_with(|| sanitize_names(o.get("name").unwrap().to_string()));
                      }
                    }
                    Ok(orgs)
                  },
                  _ => Err(anyhow!("Failed to parse json")),
                }
              },
              StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
                let msg: String = match r.json::<serde_json::Value>() {
                  Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                  _ => "unknown".to_string(),
                };
                Err(anyhow!("Auth failed: {}", msg))
              },
              _ => {
                let msg: String = match r.json::<serde_json::Value>() {
                  Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                  _ => "unknown".to_string(),
                };
                error!("Unknown request error: {}", msg);
                Err(anyhow!("Unknown request error: {}", msg))
              },
            }
          },
          Err(e) => {
            error!("Request error: {:?}", e);
            Err(anyhow!("Request error: {:?}", e))
          },
        }
      } else {
        error!("Missing auth credentials");
        Err(anyhow!("Missing auth credentials"))
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }

  pub fn set_org(&mut self, id: String) -> Result<(), anyhow::Error> {
    trace!("set_org");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(5)).connection_verbose(true).build() {
      let req_url = format!("{ip}/api/user/using/{id}", ip = self.ip.unwrap(), id = id);
      trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

      let req = c.post(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
      match req.send() {
        Ok(r) => {
          trace!("resp:{:#?}", r);
          match r.status() {
            StatusCode::OK => {
              match r.json::<serde_json::Value>() {
                Ok(t) => {
                  trace!("resp.json: {:#?}", t);
                  let msg = t.get("message").map(|m| m.to_string().replace('"', ""));
                  info!("{} = {}", msg.unwrap(), id);
                  Ok(())
                },
                _ => Err(anyhow!("Failed to parse json")),
              }
            },
            _ => {
              let msg: String = match r.json::<serde_json::Value>() {
                Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                _ => "unknown".to_string(),
              };
              error!("Unknown request error: {} / org_id: {}", msg, id);
              Err(anyhow!("Unknown request error: {} / org_id: {}", msg, id))
            },
          }
        },
        Err(e) => {
          error!("Request error: {:?}", e);
          Err(anyhow!("Request error: {:?}", e))
        },
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }

  pub fn search_dashboards(&mut self) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    trace!("search_dashboards");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(15)).connection_verbose(true).build() {
      let req_url = format!("{ip}/api/search?type=dash-db", ip = self.ip.unwrap());
      trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

      let req = c.get(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
      match req.send() {
        Ok(r) => {
          trace!("resp:{:#?}", r);
          match r.status() {
            StatusCode::OK => {
              match r.json::<serde_json::Value>() {
                Ok(t) => {
                  trace!("resp.json:{:#?}", t);
                  let mut orgs: HashMap<String, serde_json::Value> = HashMap::new();
                  for v in t.as_array().into_iter() {
                    for o in v.iter() {
                      if let Some(uid) = o.get("uid") {
                        orgs.entry(uid.to_string().replace('"', "")).or_insert_with(|| o.clone());
                      } else {
                        warn!("Failed to find the uid of the dashboards, make sure you are running >= 7.x")
                      }
                    }
                  }
                  Ok(orgs)
                },
                _ => Err(anyhow!("Failed to parse json")),
              }
            },
            _ => {
              let msg: String = match r.json::<serde_json::Value>() {
                Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                _ => "unknown".to_string(),
              };
              error!("Unknown request error: {}", msg);
              Err(anyhow!("Unknown request error: {}", msg))
            },
          }
        },
        Err(e) => {
          error!("Request error: {:?}", e);
          Err(anyhow!("Request error: {:?}", e))
        },
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }

  pub fn save_dashboards(&mut self, org_name: String, uid: String) -> Result<(), anyhow::Error> {
    trace!("save_dashboards");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(15)).connection_verbose(true).build() {
      let req_url = format!("{ip}/api/dashboards/uid/{uid}", ip = self.ip.unwrap(), uid = uid);
      trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

      let req = c.get(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
      match req.send() {
        Ok(r) => {
          trace!("resp:{:#?}", r);
          match r.status() {
            StatusCode::OK => {
              match r.json::<serde_json::Value>() {
                Ok(t) => {
                  let folder_title = t.get("meta").and_then(|m| m.get("folderTitle").map(|f| sanitize_names(f.to_string())));
                  let dashboard_title = t.get("dashboard").and_then(|m| m.get("title").map(|f| sanitize_names(f.to_string())));

                  if folder_title.is_some() && dashboard_title.is_some() {
                    let path = Path::new(&self.cfg_path.unwrap()).join(Path::new(&org_name));

                    let folder = path.join("dashboards").join(folder_title.unwrap());
                    fs::create_dir_all(&folder).expect("Cannot create dir");
                    let file = folder.join(dashboard_title.unwrap()).with_extension("json");

                    info!("Saving dashboard: {:?}", file);
                    let writer = BufWriter::new(File::create(file).expect("Cannot create file"));
                    serde_json::to_writer_pretty(writer, &t).expect("Cannot write data to file");
                  }
                  Ok(())
                },
                _ => Err(anyhow!("Failed to parse json")),
              }
            },
            _ => {
              let msg: String = match r.json::<serde_json::Value>() {
                Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                _ => "unknown".to_string(),
              };
              error!("Unknown request error: {}", msg);
              Err(anyhow!("Unknown request error: {}", msg))
            },
          }
        },
        Err(e) => {
          error!("Rrequest error: {:?}", e);
          Err(anyhow!("Request error: {:?}", e))
        },
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }

  pub fn search_datasources(&mut self) -> Result<HashMap<String, serde_json::Value>, anyhow::Error> {
    trace!("search_datasources");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(15)).connection_verbose(true).build() {
      let req_url = format!("{ip}/api/datasources", ip = self.ip.unwrap());
      trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

      let req = c.get(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
      match req.send() {
        Ok(r) => {
          trace!("resp:{:#?}", r);
          match r.status() {
            StatusCode::OK => {
              match r.json::<serde_json::Value>() {
                Ok(t) => {
                  trace!("resp.json:{:#?}", t);
                  let mut orgs: HashMap<String, serde_json::Value> = HashMap::new();
                  for v in t.as_array().into_iter() {
                    for o in v.iter() {
                      if let Some(uid) = o.get("uid") {
                        orgs.entry(uid.to_string().replace('"', "")).or_insert_with(|| o.clone());
                      } else {
                        warn!("Failed to find the uid of the datasources, make sure you are running >= 8.x")
                      }
                    }
                  }
                  Ok(orgs)
                },
                _ => Err(anyhow!("Failed to parse json")),
              }
            },
            _ => {
              let msg: String = match r.json::<serde_json::Value>() {
                Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                _ => "unknown".to_string(),
              };
              error!("Unknown request error: {}", msg);
              Err(anyhow!("Unknown request error: {}", msg))
            },
          }
        },
        Err(e) => {
          error!("Request error: {:?}", e);
          Err(anyhow!("Request error: {:?}", e))
        },
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }

  pub fn save_datasources(&mut self, org_name: String, uid: String) -> Result<(), anyhow::Error> {
    trace!("save_datasources");
    if let Ok(c) = reqwest::blocking::Client::builder().user_agent(env!("CARGO_PKG_NAME")).danger_accept_invalid_certs(true).timeout(Duration::from_secs(15)).connection_verbose(true).build() {
      let req_url = format!("{ip}/api/datasources/uid/{uid}", ip = self.ip.unwrap(), uid = uid);
      trace!("Auth on {:?} with {:?}/{:?}", req_url, self.auth_usr, self.auth_pwd);

      let req = c.get(req_url).basic_auth(&self.auth_usr.unwrap(), Some(&self.auth_pwd.unwrap()));
      match req.send() {
        Ok(r) => {
          trace!("resp:{:#?}", r);
          match r.status() {
            StatusCode::OK => {
              match r.json::<serde_json::Value>() {
                Ok(t) => {
                  let ds_type = t.get("type").map(|i| sanitize_names(i.to_string()));
                  let ds_name = t.get("name").map(|i| sanitize_names(i.to_string()));
                  if ds_type.is_some() && ds_name.is_some() {
                    let path = Path::new(&self.cfg_path.unwrap()).join(Path::new(&org_name));

                    let folder = path.join("datasources").join(ds_type.unwrap());
                    fs::create_dir_all(&folder).expect("Cannot create dir");
                    let file = folder.join(ds_name.unwrap()).with_extension("json");

                    info!("Saving datasource: {:?}", file);
                    let writer = BufWriter::new(File::create(file).expect("Cannot create file"));
                    serde_json::to_writer_pretty(writer, &t).expect("Cannot write data to file");
                  }
                  Ok(())
                },
                _ => Err(anyhow!("Failed to parse json")),
              }
            },
            _ => {
              let msg: String = match r.json::<serde_json::Value>() {
                Ok(Value::Object(m)) => m.get("message").map(|m| m.to_string().replace('"', "")).unwrap_or_else(|| "unknown".to_string()),
                _ => "unknown".to_string(),
              };
              error!("Unknown request error: {}", msg);
              Err(anyhow!("Unknown request error: {}", msg))
            },
          }
        },
        Err(e) => {
          error!("Request error: {:?}", e);
          Err(anyhow!("Request error: {:?}", e))
        },
      }
    } else {
      Err(anyhow!("Cant build client"))
    }
  }
}

fn sanitize_names(s: String) -> String { s.replace(&['/', '\\', '(', ')', '{', '}', '[', ']', ',', '\"', '.', ';', ':', '\'', '`', '!', '@', '#', '$', '%', '^', '&', '*', '~'][..], "") }
