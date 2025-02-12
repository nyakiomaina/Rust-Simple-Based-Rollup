use crate::tx::{PublicKey, Transaction};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Clone)]
pub struct Message {
    pub user_id: String,
    pub contents: String,
}

pub struct State {
    users: HashMap<PublicKey, String>,
    channels: HashMap<String, Vec<Message>>,
}

impl State {
    pub fn new() -> Self {
        State {
            users: HashMap::new(),
            channels: HashMap::new(),
        }
    }

    pub fn read_channel(&self, channel: String) -> Option<&Vec<Message>> {
        self.channels.get(&channel)
    }

    pub fn list_channels(&self) -> Vec<&String> {
        self.channels.keys().collect()
    }

    pub fn validate_tx(&self, tx: Transaction) -> Result<()> {
        let res = tx
            .clone()
            .signature()
            .verify(&tx.pubkey(), &bincode::serialize(&tx.without_signature())?);
        if !res {
            return Err(anyhow!("signature verification failed"));
        }
        match tx {
            Transaction::SendMessage(contents) => {
                if !self.users.contains_key(&contents.user) {
                    return Err(anyhow!("user not yet registered"));
                }
            }
            Transaction::Register(contents) => {
                if self.users.contains_key(&contents.user) {
                    return Err(anyhow!("user already exists"));
                }
            }
        }
        Ok(())
    }

    pub fn process_tx(&mut self, tx: Transaction) -> Result<()> {
        self.validate_tx(tx.clone())?;

        match tx {
            Transaction::SendMessage(contents) => {
                let messages = self.channels.get_mut(&contents.channel);
                let user = self.users.get(&contents.user).unwrap();

                let msg = Message {
                    user_id: user.clone(),
                    contents: contents.contents,
                };

                match messages {
                    Some(msgs) => msgs.push(msg),
                    None => {
                        self.channels.insert(contents.channel, vec![msg]);
                    }
                }
            }
            Transaction::Register(contents) => {
                if self.users.contains_key(&contents.user) {
                    return Err(anyhow!("user already exists"));
                }

                self.users.insert(contents.user, contents.id);
            }
        }

        Ok(())
    }
}