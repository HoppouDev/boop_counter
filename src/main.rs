// mod keybinding;
mod service;

use std::{
    net::{IpAddr, Ipv4Addr},
    path::PathBuf,
    str::FromStr,
    sync::Arc,
};

use serde::{Deserialize, Serialize};
use tokio::{
    fs::OpenOptions,
    io::{AsyncReadExt, AsyncWriteExt},
    sync::Mutex,
};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use vrchat_osc::{VRChatOSC, models::OscRootNode};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let filter = tracing_subscriber::EnvFilter::new("info,vrchat_osc::mdns::utils=off");

    tracing_subscriber::registry()
        .with(filter)
        .with(tracing_subscriber::fmt::layer())
        .init();

    let address: IpAddr = Ipv4Addr::from_str("127.0.0.1")?.into();
    let mut app = App::new(Some(address)).await?;

    app.run().await?;

    info!("Press Ctrl+C to exit.");
    tokio::signal::ctrl_c().await?;

    Ok(())
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct BoopCounterStateFile {
    pub boops: u32,
}

#[derive(Clone, Debug)]
pub struct BoopCounterState {
    pub boops: u32,
    pub previous_state: bool,
    pub file_path: PathBuf,
    pub message_id: u64,
    pub previous_message_id: u64,
}

impl BoopCounterState {
    pub async fn new() -> anyhow::Result<Self> {
        let file_path = PathBuf::from("state.json");

        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&file_path)
            .await?;

        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;

        let boops = serde_json::from_str::<BoopCounterStateFile>(&contents)
            .map(|s| s.boops)
            .unwrap_or(0);

        Ok(Self {
            boops,
            previous_state: false,
            file_path,
            message_id: u64::MIN,
            previous_message_id: u64::MIN,
        })
    }

    pub fn increment_message_id(&mut self) {
        self.previous_message_id = self.message_id;

        if self.message_id == u64::MAX {
            self.message_id = u64::MIN;
        } else {
            self.message_id += 1;
        }
    }

    pub fn handle_input(&mut self, state: bool) -> bool {
        let triggered = state && !self.previous_state;

        if triggered {
            self.boops += 1;
        }

        self.previous_state = state;
        triggered
    }

    pub async fn save(&self) -> anyhow::Result<()> {
        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(&self.file_path)
            .await?;

        let data = serde_json::to_vec(&BoopCounterStateFile { boops: self.boops })?;

        file.write_all(&data).await?;
        file.flush().await?;

        Ok(())
    }
}

pub struct App {
    vrchat_osc: Arc<VRChatOSC>,
    state: Arc<Mutex<BoopCounterState>>,
}

impl App {
    pub async fn new(address: Option<IpAddr>) -> anyhow::Result<Self> {
        Ok(Self {
            vrchat_osc: VRChatOSC::new(address).await?,
            state: Arc::new(Mutex::new(BoopCounterState::new().await?)),
        })
    }

    pub async fn run(&mut self) -> anyhow::Result<()> {
        self.start_chatbox_updater().await?;
        self.register_service().await?;

        Ok(())
    }

    async fn start_chatbox_updater(&mut self) -> anyhow::Result<()> {
        let osc = Arc::clone(&self.vrchat_osc);
        let state = Arc::clone(&self.state);

        tokio::spawn(async move {
            if let Err(e) = service::chatbox_updater(osc, state).await {
                error!("Chatbox updater failure: {}", e);
            }
        });

        Ok(())
    }

    async fn register_service(&mut self) -> anyhow::Result<()> {
        let root = OscRootNode::new().with_avatar();
        let osc = Arc::clone(&self.vrchat_osc);
        let state = Arc::clone(&self.state);

        osc.register("boop_counter", root, move |packet| {
            let state = Arc::clone(&state);

            tokio::spawn(async move {
                if let Err(e) = service::boop_counter(packet, state).await {
                    error!("Boop counter service failure: {}", e);
                }
            });
        })
        .await?;

        Ok(())
    }
}
