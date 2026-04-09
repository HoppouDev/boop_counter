use crate::BoopCounterState;
use rdev::{EventType, Key};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;
use vrchat_osc::rosc::{OscMessage, OscPacket};
use vrchat_osc::{VRChatOSC, rosc};

pub async fn boop_counter(
    packet: OscPacket,
    state: Arc<Mutex<BoopCounterState>>,
) -> anyhow::Result<()> {
    let mut state = state.lock().await;

    let message: OscMessage = match packet {
        OscPacket::Message(msg) => msg,
        OscPacket::Bundle(_) => {
            return Err(anyhow::Error::msg(
                "Received unexpected OSC bundle on boop handler",
            ));
        }
    };

    if message.addr != "/avatar/parameters/Boop" {
        return Ok(());
    }

    let Some(rosc::OscType::Bool(value)) = message.args.first() else {
        return Ok(());
    };

    state.increment_message_id();

    if state.handle_input(*value) {
        if let Err(e) = state.save().await {
            anyhow::bail!("Failed to save state: {e}");
        }
    }

    Ok(())
}

pub async fn chatbox_updater(
    osc: Arc<VRChatOSC>,
    state: Arc<Mutex<BoopCounterState>>,
) -> anyhow::Result<()> {
    loop {
        let (message_id, previous_message_id, boops) = {
            let state = state.lock().await;
            (state.message_id, state.previous_message_id, state.boops)
        };

        if message_id != previous_message_id {
            state.lock().await.previous_message_id = message_id;

            let _ = osc
                .send(
                    OscPacket::Message(OscMessage {
                        addr: "/chatbox/input".to_string(),
                        args: vec![
                            rosc::OscType::String(format!("Times Booped: {}\n(Automatic)", boops)),
                            rosc::OscType::Bool(true),
                        ],
                    }),
                    "VRChat-Client-*",
                )
                .await;
        }

        tokio::time::sleep(Duration::from_secs_f32(2.0)).await;
    }
}

pub async fn _capture_hypnosis(_osc: Arc<VRChatOSC>) -> anyhow::Result<()> {
    if let Err(e) = rdev::listen(|event: rdev::Event| match event.event_type {
        EventType::KeyPress(key) => match key {
            Key::RightArrow => {}
            _ => {}
        },
        _ => {}
    }) {
        anyhow::bail!("Failed to listen to input events: {:?}", e);
    }

    Ok(())
}

// async fn keybind_callback_listener() {}
