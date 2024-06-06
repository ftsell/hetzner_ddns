use core::fmt;
use std::{
    net::{IpAddr, Ipv4Addr, Ipv6Addr},
    path::{Path, PathBuf},
    str::FromStr,
};

use clap::Parser;
use eyre::{eyre, Context};
use hetzner_dns::Client;
use reqwest::Url;
use serde::Deserialize;
use tokio::fs;
use tracing::Level;

mod hetzner_dns;

#[derive(Debug, Deserialize)]
struct Target {
    #[serde(rename = "zone")]
    zone_name: String,
    #[serde(rename = "record")]
    record_name: String,
}

#[derive(Deserialize)]
struct Config {
    api_token: String,
    targets: Vec<Target>,
}

impl fmt::Debug for Config {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Config")
            .field("api_token", &"**********")
            .field("targets", &self.targets)
            .finish()
    }
}

#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Path to a config.toml file
    #[arg(short = 'c', long = "config")]
    config: PathBuf,

    /// Emit more verbose output
    #[arg(short = 'v', long = "verbose")]
    debug: bool,
}

type OwnAddrs = (Option<Ipv4Addr>, Option<Ipv6Addr>);

#[tokio::main(flavor = "current_thread")]
async fn main() -> eyre::Result<()> {
    color_eyre::install().unwrap();
    let cli = Cli::parse();
    tracing_subscriber::fmt::fmt()
        .with_max_level(match cli.debug {
            true => Level::DEBUG,
            false => Level::INFO,
        })
        .compact()
        .init();
    let config = read_config(&cli.config)
        .await
        .expect("Could not read config");

    let ips = get_own_ips().await?;

    let mut req_client = Client::new(&config.api_token);
    req_client
        .get_all_zones(None, None, None)
        .await
        .with_context(|| "Api-Key does not seem valid since no zones could be listed")?;

    for zone in &config.targets {
        update_zone(&mut req_client, &zone, &ips).await?;
    }

    Ok(())
}

async fn read_config(path: &Path) -> eyre::Result<Config> {
    tracing::debug!("Reading config from {}", path.display());
    let file = fs::read_to_string(path)
        .await
        .with_context(|| format!("Could not read string data from {}", path.display()))?;
    let config = toml::from_str(&file)?;
    tracing::debug!("Successfully read config: {:#?}", config);
    Ok(config)
}

async fn update_zone(client: &mut Client, zone: &Target, own_addrs: &OwnAddrs) -> eyre::Result<()> {
    let records = find_records(client, zone).await?;
    for i_record in records {
        let value = match i_record.typ.as_str() {
            "A" => match own_addrs.0 {
                Some(ip) => ip.to_string(),
                None => {
                    tracing::warn!(
                        "Cannot update A record {} becacuse host has no IPv4 connectivity",
                        i_record.name
                    );
                    continue;
                }
            },
            "AAAA" => match own_addrs.1 {
                Some(ip) => ip.to_string(),
                None => {
                    tracing::warn!(
                        "Cannot update AAAA record {} because host has no IPv6 connectivity",
                        i_record.name
                    );
                    continue;
                }
            },
            _ => unreachable!("records other than A and AAAA are filtered out beforehand"),
        };

        tracing::info!("Updating record {} to {}", i_record.name, value);
        client
            .update_record(
                &i_record.id,
                &hetzner_dns::UpdateRecordData {
                    name: i_record.name,
                    ttl: 60,
                    typ: i_record.typ,
                    zone_id: i_record.zone_id,
                    value: value,
                },
            )
            .await?;
    }

    Ok(())
}

async fn find_records(
    client: &mut Client,
    target: &Target,
) -> eyre::Result<Vec<hetzner_dns::Record>> {
    tracing::debug!("Searching hetzner record for {:?}", target);
    let htz_zone = client
        .get_all_zones(Some(&target.zone_name), None, None)
        .await
        .with_context(|| format!("Could not retrieve information about zone {}. Ensure that it exists and you have permission to access it", &target.zone_name))?
        .content
        .zones
        .first()
        .unwrap()
        .to_owned();

    let htz_records = client
        .get_all_records(&htz_zone.id)
        .await?
        .records
        .into_iter()
        .filter(|i_record| i_record.name == target.record_name)
        .filter(|i_record| i_record.typ == "A" || i_record.typ == "AAAA")
        .collect::<Vec<_>>();

    tracing::debug!("Found target records {:?}", htz_records);
    Ok(htz_records)
}

async fn get_own_ips() -> eyre::Result<OwnAddrs> {
    tracing::debug!("Fetching own ip addresses from ip.kritzl.dev");
    let ipv4 = match reqwest::get(Url::parse("https://4.kritzl.dev").unwrap()).await {
        Err(_) => None,
        Ok(response) => Some(
            Ipv4Addr::from_str(&response.text().await?)
                .with_context(|| "ip.kritzl.dev did not return a well-formed IPv4 address")?,
        ),
    };

    let ipv6 = match reqwest::get(Url::parse("https://6.kritzl.dev").unwrap()).await {
        Err(_) => None,
        Ok(response) => Some(
            Ipv6Addr::from_str(&response.text().await?)
                .with_context(|| "ip.kritzl.dev did not return a well-formed IPv6 address")?,
        ),
    };

    match (ipv4, ipv6) {
        (None, None) => {
            return Err(eyre!(
                "ip.kritzl.dev did not return any ip addresses but we were able to reach it"
            ))
        }
        (Some(ipv4), None) => {
            tracing::info!("Host has IPv4 connectivity from {} but no IPv6", ipv4)
        }
        (None, Some(ipv6)) => {
            tracing::info!("Host has IPv6 connectivity from {} but no IPv4", ipv6)
        }
        (Some(ipv4), Some(ipv6)) => tracing::info!(
            "Host has dual-stack connectivity from {} and {}",
            ipv6,
            ipv4
        ),
    };

    Ok((ipv4, ipv6))
}
