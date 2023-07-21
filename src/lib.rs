mod configs;

use bevy::{prelude::*, tasks::AsyncComputeTaskPool};
use dojo_client::contract::world::WorldContract;
use rand::Rng;
use starknet::{
    accounts::SingleOwnerAccount,
    {
        core::{
            types::{BlockId, BlockTag, FieldElement},
            utils::cairo_short_string_to_felt,
        },
        providers::{jsonrpc::HttpTransport, JsonRpcClient},
        signers::{LocalWallet, SigningKey},
    },
};
use std::str::FromStr;
use url::Url;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    // // The `console.log` is quite polymorphic, so we can bind it with multiple
    // // signatures. Note that we need to use `js_name` to ensure we always call
    // // `log` in JS.
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_u32(a: u32);

    // // Multiple arguments too!
    // #[wasm_bindgen(js_namespace = console, js_name = log)]
    // fn log_many(a: &str, b: &str);
}

#[wasm_bindgen(start)]
pub fn main() {
    log("Starting bevy app...");

    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, (ping_github, spawn_racers))
        .run();
}

fn ping_github() {
    let thread_pool = AsyncComputeTaskPool::get();

    thread_pool
        .spawn(async move {
            match reqwest::Client::new()
                .get("https://api.github.com/repos/rustwasm/wasm-bindgen/branches/master")
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await
            {
                Ok(res) => {
                    match res.text().await {
                        Ok(text) => {
                            log(&format!("res: {text:#?}"));

                            // let branch_info: Branch = serde_json::from_str(&text).unwrap();
                            // Ok(JsValue::from_serde(&branch_info).unwrap())
                        }
                        Err(e) => {
                            log(&format!("Failed to parse response as text: {e}"));
                        }
                    }
                }
                Err(e) => {
                    log(&format!("Request failed: {e}"));
                }
            }
        })
        .detach();
}

fn spawn_racers() {
    let thread_pool = AsyncComputeTaskPool::get();

    thread_pool
        .spawn(async move {
            let url = Url::parse(configs::JSON_RPC_ENDPOINT).unwrap();
            let account_address = FieldElement::from_str(configs::ACCOUNT_ADDRESS).unwrap();
            let account = SingleOwnerAccount::new(
                JsonRpcClient::new(HttpTransport::new(url)),
                LocalWallet::from_signing_key(SigningKey::from_secret_scalar(
                    FieldElement::from_str(configs::ACCOUNT_SECRET_KEY).unwrap(),
                )),
                account_address,
                cairo_short_string_to_felt("KATANA").unwrap(),
            );

            let world_address = FieldElement::from_str(configs::WORLD_ADDRESS).unwrap();

            let world = WorldContract::new(world_address, &account);
            let block_id = BlockId::Tag(BlockTag::Latest);

            let spawn_racer_system = world.system("spawn_racer", block_id).await.unwrap();
            let model_id = cairo_short_string_to_felt(configs::MODEL_NAME).unwrap();

            match spawn_racer_system
                .execute(vec![
                    model_id,
                    rand_felt_fixed_point(),
                    FieldElement::ZERO,
                    FieldElement::ZERO,
                    FieldElement::ZERO,
                ])
                .await
            {
                Ok(_) => {
                    log("success");
                }
                Err(_) => {
                    // log::error!("Run spawn_racer system: {e}");
                }
            }
        })
        .detach();
}

fn rand_felt_fixed_point() -> FieldElement {
    let mut rng = rand::thread_rng();
    ((rng.gen::<u128>() % 200) << 64).into()
}
