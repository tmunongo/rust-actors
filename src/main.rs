use std::{collections::HashMap, io::stdin};

use actix::prelude::*;

#[derive(Message)]
#[rtype(result = "usize")]
pub struct AddWord(pub String);
#[derive(Message)]
#[rtype(result = "HashMap<String, u64>")]
pub struct GetCounts;

pub struct WordCountAggregator {
    counts: HashMap<String, u64>,
}

impl Actor for WordCountAggregator {
    type Context = Context<Self>;
}

impl Handler<AddWord> for WordCountAggregator {
    type Result = usize;

    fn handle(&mut self, message: AddWord, _: &mut Self::Context) -> Self::Result {
        let AddWord(word) = message;
        let count = self.counts.entry(word.clone()).or_insert(0);
        *count += 1;
        println!("Added word: {}, count is now {}", word, count);
        *count as usize
    }
}

impl Handler<GetCounts> for WordCountAggregator {
    type Result = ResponseActFuture<Self, HashMap<String, u64>>;

    fn handle(&mut self, _msg: GetCounts, _ctx: &mut Self::Context) -> Self::Result {
        let counts = self.counts.clone();
        let fut = async move { counts };
        Box::pin(fut.into_actor(self))
    }
}

#[derive(Clone)]
pub struct AggregatorHandle {
    addr: Addr<WordCountAggregator>,
}

impl AggregatorHandle {
    pub fn new() -> Self {
        let actor = WordCountAggregator {
            counts: HashMap::new(),
        };

        let addr = actor.start();
        AggregatorHandle { addr }
    }

    // actors referenced by address
    pub async fn add_word(&self, word: String) {
        let _ = self.addr.send(AddWord(word)).await;
    }

    pub async fn get_counts(&self) -> Result<HashMap<String, u64>, actix::MailboxError> {
        self.addr.send(GetCounts).await
    }
}

#[actix::main]
async fn main() {
    let handle = AggregatorHandle::new();
    loop {
        let mut input = String::new();
        println!("Enter a word:");
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();
        if input == "quit" {
            break;
        }
        let _ = handle.add_word(input.to_string()).await;
    }
    match handle.get_counts().await {
        Ok(counts) => {
            println!("Final counts: {:?}", counts);
        }
        Err(e) => {
            println!("Error getting counts: {:?}", e);
        }
    }
}
