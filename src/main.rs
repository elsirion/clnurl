//! A mostly reverse-engineered implementation of LNURLPay following <https://bolt.fun/guide/web-services/lnurl/pay>

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use cln_plugin::options::{ConfigOption, Value};
use cln_rpc::model::InvoiceRequest;
use cln_rpc::primitives::AmountOrAny;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::io::{stdin, stdout};
use url::Url;
use uuid::Uuid;

use nostr::event::Event;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let plugin = if let Some(plugin) = cln_plugin::Builder::new(stdin(), stdout())
        .option(ConfigOption::new(
            "clnurl_listen",
            Value::String("127.0.0.1:9876".into()),
            "Listen address for the LNURL web server",
        ))
        .option(ConfigOption::new(
            "clnurl_base_address",
            Value::String("http://localhost/".into()),
            "Base path under which the API endpoints are reachable, e.g. \
            https://example.com/lnurl_api means endpoints are reachable as \
            https://example.com/lnurl_api/lnurl and https://example.com/lnurl_api/invoice",
        ))
        .option(ConfigOption::new(
            "clnurl_description",
            Value::String("Gimme money!".into()),
            "Description to be displayed in LNURL",
        ))
        .option(ConfigOption::new(
            "clnurl_nostr_pubkey",
            Value::OptString,
            "Nostr pub key of zapper",
        ))
        .dynamic()
        .start(())
        .await?
    {
        plugin
    } else {
        return Ok(());
    };

    let rpc_socket: PathBuf = plugin.configuration().rpc_file.parse()?;
    let listen_addr: SocketAddr = plugin
        .option("clnurl_listen")
        .expect("Option is defined")
        .as_str()
        .expect("Option is a string")
        .parse()?;

    let api_base_address: Url = plugin
        .option("clnurl_base_address")
        .expect("Option is defined")
        .as_str()
        .expect("Option is a string")
        .parse()?;

    let description = plugin
        .option("clnurl_description")
        .expect("Option is defined")
        .as_str()
        .expect("Option is a string")
        .to_owned();

    let nostr_pubkey = plugin
        .option("clnurl_nostr_pubkey")
        .map(|v| v.as_str().expect("Option is string").to_owned());

    let state = ClnurlState {
        rpc_socket,
        api_base_address,
        description,
        nostr_pubkey,
    };

    let lnurl_service = Router::new()
        .route("/lnurl", get(get_lnurl_struct))
        .route("/invoice", get(get_invoice))
        .with_state(state);

    axum::Server::bind(&listen_addr)
        .serve(lnurl_service.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Clone)]
struct ClnurlState {
    rpc_socket: PathBuf,
    api_base_address: Url,
    description: String,
    nostr_pubkey: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LnurlResponse {
    // TODO: introduce amount type, figure out if this is sat or msat
    min_sendable: u64,
    // TODO: introduce amount type, figure out if this is sat or msat
    max_sendable: u64,
    metadata: String,
    callback: Url,
    tag: LnurlTag,
    allow_nostr: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    nostr_pubkey: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
enum LnurlTag {
    PayRequest,
}

async fn get_lnurl_struct(
    State(state): State<ClnurlState>,
) -> Result<Json<LnurlResponse>, StatusCode> {
    Ok(Json(LnurlResponse {
        min_sendable: 0,
        max_sendable: 100000000000,
        metadata: serde_json::to_string(&vec![vec!["text/plain".to_string(), state.description]])
            .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?,
        callback: state
            .api_base_address
            .join("invoice")
            .expect("Still a valid URL"),
        tag: LnurlTag::PayRequest,
        allow_nostr: state.nostr_pubkey.is_some(),
        nostr_pubkey: state.nostr_pubkey,
    }))
}

#[derive(Deserialize)]
struct GetInvoiceParams {
    // TODO: introduce amount type, figure out if this is sat or msat
    amount: u64,
    nostr: Option<String>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GetInvoiceResponse {
    pr: String,
    // TODO: find out proper type
    success_action: Option<String>,
    // TODO: find out proper type
    routes: Vec<String>,
}

async fn get_invoice(
    Query(params): Query<GetInvoiceParams>,
    State(state): State<ClnurlState>,
) -> Result<Json<GetInvoiceResponse>, StatusCode> {
    let mut cln_client = cln_rpc::ClnRpc::new(&state.rpc_socket)
        .await
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let description = match &params.nostr {
        Some(d) => {
            let zap_request: Event =
                Event::from_json(d).map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
            zap_request
                .verify()
                .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;
            zap_request.as_json()
        }
        None => state.description,
    };

    let cln_response = cln_client
        .call(cln_rpc::Request::Invoice(InvoiceRequest {
            amount_msat: AmountOrAny::Amount(cln_rpc::primitives::Amount::from_msat(params.amount)),
            description,
            label: Uuid::new_v4().to_string(),
            expiry: None,
            fallbacks: None,
            preimage: None,
            exposeprivatechannels: None,
            cltv: None,
            deschashonly: Some(true),
        }))
        .await
        .map_err(|_e| StatusCode::INTERNAL_SERVER_ERROR)?;

    let invoice = match cln_response {
        cln_rpc::Response::Invoice(invoice_response) => invoice_response.bolt11,
        _ => panic!("CLN returned wrong response kind"),
    };

    Ok(Json(GetInvoiceResponse {
        pr: invoice,
        success_action: None,
        routes: vec![],
    }))
}
