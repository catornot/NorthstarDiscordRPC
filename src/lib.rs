#![allow(non_snake_case)]

use discord_sdk::activity::Secrets;
use parking_lot::Mutex;
use rrplug::{bindings::cvar::convar::FCVAR_CLIENTDLL, prelude::*};
use tokio::runtime::Runtime;
use uuid::Uuid;

use crate::{
    discord::async_main,
    presence::run_presence_updates,
    presense_bindings::{GameState, GameStateStruct, UIPresenceStruct},
};

pub(crate) mod discord;
pub(crate) mod presence;
pub(crate) mod presense_bindings;

#[deny(non_snake_case)]
#[derive(Debug, Default, Clone)]
#[doc = "struct for all the possible information on the rpc"]
pub struct ActivityData {
    party: Option<(u32, u32)>,
    details: String,
    state: String,
    large_image: Option<String>,
    large_text: Option<String>,
    small_image: Option<String>,
    small_text: Option<String>,
    end: Option<i64>,
    start: Option<i64>,
    last_state: GameState,
    secrets: Secrets,
}

#[deny(non_snake_case)]
pub struct DiscordRpcPlugin {
    pub activity: Mutex<ActivityData>,
    pub presence_data: Mutex<(GameStateStruct, UIPresenceStruct)>,
}

#[deny(non_snake_case)]
impl Plugin for DiscordRpcPlugin {
    const PLUGIN_INFO: PluginInfo = PluginInfo::new(
        "DISCORDRPC\0",
        "DISCORDXD\0",
        "DISCORDRPC\0",
        PluginContext::CLIENT,
    );

    fn new(_: bool) -> Self {
        register_sq_functions(presence::fetch_presence);

        let activity = Mutex::new(ActivityData {
            large_image: Some("northstar".to_string()),
            state: "Loading...".to_string(),
            ..Default::default()
        });

        std::thread::spawn(|| match Runtime::new() {
            Ok(rt) => rt.block_on(async_main()),
            Err(err) => {
                log::error!("failed to create a runtime; {:?}", err);
                return;
            }
        });

        Self {
            activity,
            presence_data: Mutex::new((GameStateStruct::default(), UIPresenceStruct::default())),
        }
    }

    fn on_sqvm_created(&self, sqvm_handle: &CSquirrelVMHandle, _: EngineToken) {
        match sqvm_handle.get_context() {
            ScriptContext::CLIENT | ScriptContext::UI => {
                run_presence_updates(unsafe { sqvm_handle.get_sqvm() })
            }
            _ => {}
        }
    }

    fn on_dll_load(
        &self,
        engine_data: Option<&EngineData>,
        _dll_ptr: &DLLPointer,
        _engine_token: EngineToken,
    ) {
        let Some(engine_data) = engine_data else {
            return;
        };

        engine_data
            .register_concommand(
                "discord_rpc_reload_secrets",
                reload_secrets,
                "reloads the secrets for joining games",
                FCVAR_CLIENTDLL as i32,
            )
            .expect("couldn't register reload secrets");
    }
}

#[rrplug::concommand]
fn reload_secrets() {
    PLUGIN.wait().activity.lock().secrets = Secrets {
        r#match: Uuid::new_v4().to_string().into(),
        join: Uuid::new_v4().to_string().into(),
        spectate: None,
    };
}

entry!(DiscordRpcPlugin);
