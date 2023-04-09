use core::panic;
use std::collections::BTreeMap;
use base64::engine::Config;
use serde::Deserialize;

use tendermint_rpc::{WebSocketClient, SubscriptionClient};
use tendermint_rpc::query::{EventType, Query};
use futures::StreamExt;
use tendermint_rpc::event::EventData;
use base64::engine::general_purpose::STANDARD;

// const SUBSCRIBE_TX: &str = r#"{ "jsonrpc": "2.0", "method": "subscribe", "params": ["tm.event='Tx'"], "id": 2 }"#;


#[derive(Debug, Clone)]
pub struct BaseTransaction {
    /// `[ "F0E26D70191E27C8AB6249DE9C088B8C2812443CDF0DF04D7C83AE76A117C083" ]`
    // #[serde(rename = "tx.hash")]
    pub tx_hash: String,

    /// `[ "2931697000000000aevmos" ]`
    // #[serde(rename = "tx.fee")]
    pub tx_fee: String,

    /// `[ "8076531" ]`
    // #[serde(rename = "tx.height")]
    pub tx_height: String,

    /// `[ "/ethermint.evm.v1.MsgEthereumTx" ]`
    // #[serde(rename = "message.action")]
    pub message_action: String,

    /// `[ "1535902500000000aevmos" ]`
    // #[serde(rename = "transfer.amount")]
    pub transfer_amount: String,
}

impl BaseTransaction {
    fn from_tx_events(ev: TXMap) -> Self {


        Self {
            tx_hash: ev["tx.hash"].get(0).unwrap().to_string(),
            tx_fee: ev["tx.fee"].get(0).unwrap().to_string(),
            tx_height: ev["tx.height"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
            transfer_amount: ev["transfer.amount"].get(0).unwrap().to_string(),

        }
    }
}

pub type TXMap = BTreeMap<String, Vec<String>>;

#[derive(Debug, Clone)]
pub struct ConfirmDepositStarted {
    chain: String,
    participants: String,
    tx_id: String,
    evm_deposit_address: String,
    action: String,
}

impl ConfirmDepositStarted {
    fn from_tx_events(ev: TXMap) -> Self {
        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmDepositStarted.chain"].get(0).unwrap().to_string(),
            participants: ev["axelar.evm.v1beta1.ConfirmDepositStarted.participants"].get(0).unwrap().to_string(),
            tx_id: ev["axelar.evm.v1beta1.ConfirmDepositStarted.tx_id"].get(0).unwrap().to_string(),
            evm_deposit_address: ev["axelar.evm.v1beta1.ConfirmDepositStarted.deposit_address"].get(0).unwrap().to_string(),
            action: ev["message.action"].get(0).unwrap().to_string()
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfirmGatewayTxStartedEvents {
    chain: String,
    participants: PollParticipants,
    tx_id: String,
    message_action: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct PollParticipants {
    poll_id: String,
    participants: Vec<String>,
}

impl ConfirmGatewayTxStartedEvents {
    fn from_tx_events(ev: TXMap) -> Self {
        let participants = serde_json::from_str(ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants"].get(0).unwrap()).unwrap();
        Self {
            chain: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.chain"].get(0).unwrap().to_string(),
            participants,
            tx_id: ev["axelar.evm.v1beta1.ConfirmGatewayTxStarted.tx_id"].get(0).unwrap().to_string(),
            message_action: ev["message.action"].get(0).unwrap().to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConfirmKeyTransferStartedEvents {
    chain: String,
    participants: String,
    tx_id: String,
    message_action: String,
}

impl ConfirmKeyTransferStartedEvents {
    fn from_tx_events(ev: TXMap) -> Self {
        Self {
        chain: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.chain"].get(0).unwrap().to_string(),
        participants: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants"].get(0).unwrap().to_string(),
        tx_id: ev["axelar.evm.v1beta1.ConfirmKeyTransferStarted.tx_id"].get(0).unwrap().to_string(),
        message_action: ev["message.action"].get(0).unwrap().to_string(),
            }
    }
}

#[derive(Debug, Clone)]
pub struct PollVoteEvent {
    poll_state: String,
}

impl PollVoteEvent {
    fn from_tx_events(ev: TXMap) -> Self {
        Self {
            poll_state: ev["axelar.vote.v1beta1.Voted.state"].get(0).unwrap().to_string()
        }
    }
}

// #[derive(Debug, Clone)]
// pub enum SpecialTransaction {
    // ProposalVoteTx { events: ProposalVoteEvents },
    // ConfirmERC20DepositStartedTx { events: ConfirmDepositStartedEvents },
    // ConfirmDepositStartedTx { events: ConfirmDepositStartedEvents },
    // ConfirmGatewayTxStartedTx { events: ConfirmGatewayTxStartedEvents },
    // ConfirmKeyTransferStartedTx { events: ConfirmKeyTransferStartedEvents },
    // VotedTx { events: VotedTxEvents },
// }

#[tokio::main]
async fn main() {
    let (client, driver) = WebSocketClient::new("wss://rpc-axelar.stakerun.com/websocket")
        .await
        .unwrap();

    let deriver_handle = tokio::spawn(async move { driver.run().await });

    // let subscribe = Query::from(EventType::Tx).and_eq(key, value)
    let mut subs = client.subscribe(EventType::Tx.into())
        .await
        .unwrap();

    while let Some(res) = subs.next().await {
        if let Ok(ev) = res {
            let events = ev.events.clone().unwrap();
            match ev.data {
                EventData::NewBlock { block, result_begin_block, result_end_block } => todo!(),
                EventData::Tx { tx_result } => {
                    let tx = BaseTransaction::from_tx_events(events.clone());

                    match tx.message_action.as_str() {
                        "ConfirmERC20Deposit" | "ConfirmDeposit" => {
                            if events.contains_key("axelar.evm.v1beta1.ConfirmDepositStarted.participants") {
                                let sp_tx = ConfirmDepositStarted::from_tx_events(events);
                                dbg!(sp_tx);
                            }
                        },
                        "ConfirmGatewayTx" => {
                            if events.contains_key("axelar.evm.v1beta1.ConfirmGatewayTxStarted.participants") {
                                let sp_tx = ConfirmGatewayTxStartedEvents::from_tx_events(events);
                                dbg!(sp_tx);
                            }
                        },
                        "ConfirmTransferKey" => {
                            if events.contains_key("axelar.evm.v1beta1.ConfirmKeyTransferStarted.participants") {
                                let sp_tx = ConfirmKeyTransferStartedEvents::from_tx_events(events);
                                dbg!(sp_tx);
                            }
                        },
                        other => {
                            if events.contains_key("axelar.vote.v1beta1.Voted.state") {
                                dbg!(other);
                                let sp_tx = PollVoteEvent::from_tx_events(events);
                                dbg!(sp_tx);
                            }
                        }
                        // m => { if m != "RefundMsgRequest" { dbg!(m); } }
                    }
                    // dbg!(tx);

                },
                EventData::GenericJsonEvent(_) => todo!(),
            }
        }
    }

    client.close().unwrap();

    let _ = deriver_handle.await.unwrap();
}
