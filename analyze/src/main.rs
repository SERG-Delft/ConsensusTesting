use analyzer::analyze;
use chrono::prelude::*;
use clap::Parser;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Deserialize;
use serialize::{RippleMessage, RippleMessageObject};
use std::{
    fs::File,
    io::{self, BufRead},
};

mod analyzer;

lazy_static! {
    static ref RE: Regex = Regex::new(r"(\d+) (\S*) \[(\d)->(\d)\] sent (\S+): (.+)").unwrap();
    static ref VALIDATION_RE: Regex = Regex::new(r#"^\{"([^"]+)":(.+)\},$"#).unwrap();
}

#[derive(Parser)]
struct Args {
    path: String,
}

fn main() {
    let args = Args::parse();
    let messages = read(args.path.clone() + "/execution.txt").unwrap();
    let subscriptions = (0..7)
        .map(|i| {
            read_subscriptions(args.path.clone() + &format!("/subscription_{}.json", i)).unwrap()
        })
        .collect();
    analyze(&messages, subscriptions);
}

fn read(path: String) -> io::Result<Vec<RippleMessage>> {
    let mut messages = vec![];

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines() {
        let line = line?;
        if line.starts_with("--") {
            continue;
        }
        let captures = RE.captures(&line).unwrap();
        let message = match &captures[5] {
            "ProposeSet" => Some(RippleMessageObject::TMProposeSet(
                serde_json::from_str(&captures[6]).unwrap(),
            )),
            "Validation" => Some(RippleMessageObject::TMValidation(
                serde_json::from_str(&captures[6]).unwrap(),
            )),
            _ => None,
        };

        if message.is_none() {
            continue;
        }

        messages.push(RippleMessage {
            from_node: captures[3].to_string(),
            to_node: captures[4].to_string(),
            timestamp: Utc.from_utc_datetime(&NaiveDateTime::from_timestamp(
                captures[1].parse::<i64>().unwrap(),
                0,
            )),
            message: message.unwrap(),
        });
    }

    Ok(messages)
}

#[derive(Deserialize)]
pub struct Validated {
    ledger_hash: String,
}

fn read_subscriptions(path: String) -> io::Result<Vec<Validated>> {
    let mut validations = vec![];

    let file = File::open(path)?;
    let reader = io::BufReader::new(file);

    for line in reader.lines().skip(1) {
        let line = line?;
        let captures = VALIDATION_RE.captures(line.as_str()).unwrap();
        if &captures[1] == "LedgerValidated" {
            validations.push(serde_json::from_str(&captures[2]).unwrap())
        }
    }

    Ok(validations)
}
