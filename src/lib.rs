mod fq_api;
mod fq_struct;

use worker::*;
use serde::{Serialize, Deserialize};
use crate::fq_api::batch_full;
use crate::fq_struct::FqVariable;

#[event(fetch)]
async fn fetch(
    req: Request,
    env: Env,
    ctx: Context,
) -> Result<Response> {
    console_error_panic_hook::set_once();
    let router = Router::new();

    router
        .get_async("/item_id/:item_id", |req, ctx| async move {
            if let Some(item_ids) = ctx.param("item_id") {
                let Ok(client) = reqwest::Client::builder().build()
                else { return Response::error("build client failed", 411) };
                let ref var = FqVariable {
                    install_id: ctx.var("install_id")?.to_string(),
                    server_device_id: ctx.var("server_device_id")?.to_string(),
                    aid: ctx.var("aid")?.to_string(),
                    update_version_code: ctx.var("update_version_code")?.to_string(),
                };
                let Ok(batch_full) = batch_full(&client, var, item_ids, false).await
                else { return Response::error("batch_full failed", 412) };
                let Ok(res) = batch_full.get_decrypt_contents(&client, var).await
                else { return Response::error("get_decrypt_contents failed", 413) };
                let line1 = res.get(0).unwrap();
                return Response::ok(line1.1.clone());
            }
            return Response::error("Bad Request", 400)
        })
        .run(req, env)
        .await
}