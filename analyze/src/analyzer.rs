use polars::prelude::*;
use serialize::parser::parse;
use serialize::{RippleMessage, RippleMessageObject};
use std::collections::{HashMap, HashSet};
use std::ops::Add;

use openssl::sha::sha256;

use crate::Validated;

pub fn public_key_to_b58(key: &[u8]) -> String {
    let type_prefixed_key = [&[28u8], key].concat();
    let checksum = sha256(&sha256(&type_prefixed_key));
    let key = [&type_prefixed_key, &checksum[..4]].concat();
    bs58::encode(key)
        .with_alphabet(bs58::Alphabet::RIPPLE)
        .into_string()
}

pub fn analyze(messages: &[RippleMessage], subscriptions: Vec<Vec<Validated>>) {
    println!("found {} messages", messages.len());

    let mut proposals: HashMap<_, HashMap<_, HashMap<_, HashSet<_>>>> = HashMap::new();
    let mut proposals_vec = vec![];
    let mut propo_recv = vec![];
    let mut val_recv = vec![];
    let mut propo_sender = vec![];
    let mut val_sender = vec![];
    let mut validations = vec![];

    let mut full_validatios_sender = vec![];
    let mut full_validations_hash = vec![];

    for (i, validations) in subscriptions.into_iter().enumerate() {
        for validation in validations {
            full_validatios_sender.push(i as u64);
            full_validations_hash.push(validation.ledger_hash.to_lowercase());
        }
    }

    for message1 in messages {
        let message = &message1.message;
        match message {
            RippleMessageObject::TMProposeSet(proposal) => {
                proposals_vec.push(proposal);
                propo_recv.push(message1.to_node.parse::<u64>().unwrap());
                propo_sender.push(message1.from_node.parse::<u64>().unwrap());
                let ledger = hex::encode(proposal.get_previousledger());
                let tx_hash = hex::encode(proposal.get_currentTxHash());
                let node = match public_key_to_b58(proposal.get_nodePubKey()).as_str() {
                    "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => 0,
                    "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => 1,
                    "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => 2,
                    "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => 3,
                    "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => 4,
                    "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => 5,
                    "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => 6,
                    _ => unreachable!(),
                };
                let seq = proposal.get_proposeSeq();

                proposals
                    .entry(ledger)
                    .or_default()
                    .entry(seq)
                    .or_default()
                    .entry(tx_hash)
                    .or_default()
                    .insert(node);
            }
            RippleMessageObject::TMValidation(validation) => {
                val_recv.push(message1.to_node.parse::<u64>().unwrap());
                val_sender.push(message1.from_node.parse::<u64>().unwrap());
                validations.push(validation);
            }
            _ => {}
        }
    }

    let (p_seq, p_prev, p_hash, p_sender) = proposals_vec.iter().fold(
        (vec![], vec![], vec![], vec![]),
        |(mut p_seq, mut p_prev, mut p_hash, mut p_sender), proposal| {
            p_seq.push(proposal.get_proposeSeq());
            p_prev.push(hex::encode(proposal.get_previousledger()));
            p_hash.push(hex::encode(proposal.get_currentTxHash()));
            p_sender.push(
                match public_key_to_b58(proposal.get_nodePubKey()).as_str() {
                    "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => 0,
                    "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => 1,
                    "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => 2,
                    "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => 3,
                    "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => 4,
                    "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => 5,
                    "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => 6,
                    _ => unreachable!(),
                },
            );
            (p_seq, p_prev, p_hash, p_sender)
        },
    );

    let (seqs, hashs, owner, consensus) = validations.iter().fold(
        (vec![], vec![], vec![], vec![]),
        |(mut x, mut y, mut owner, mut consensus), val| {
            let (_, validation) = parse(val.get_validation()).unwrap();
            x.push(validation["LedgerSequence"].as_usize().unwrap() as u64);
            consensus.push(validation["ConsensusHash"].as_str().unwrap().to_owned());
            y.push(validation["hash"].as_str().unwrap().to_owned());
            owner.push(
                match public_key_to_b58(
                    hex::decode(validation["SigningPubKey"].as_str().unwrap())
                        .unwrap()
                        .as_slice(),
                )
                .as_str()
                {
                    "n9KkgT2SFxpQGic7peyokvkXcAmNLFob1AZXeErMFHxJ71q5MGaK" => 0,
                    "n9M6ouZU7cLwRHPiVZjgJdEgrVyx2uv9euZzepdb34wDoj1RP5uS" => 1,
                    "n9LJhBqLGTjPQa2KJtJmkHUubaHs1Y1ENYKZVmzZYhNb7GXh9m4j" => 2,
                    "n9KgN4axJo1WC3fjFoUSkJ4gtZX4Pk2jPZzGR5CE9ddo16ewAPjN" => 3,
                    "n9MsRMobdfpGvpXeGb3F6bm7WZbCiPrxzc1qBPP7wQox3NJzs5j2" => 4,
                    "n9JFX46v3d3WgQW8DJQeBwqTk8vaCR7LufApEy65J1eK4X7dZbR3" => 5,
                    "n9LFueHyYVJSyDArog2qtR42NixmeGxpaqFEFFp1xjxGU9aYRDZc" => 6,
                    _ => unreachable!(),
                },
            );
            (x, y, owner, consensus)
        },
    );

    let full_validations = df![
        "hash" => full_validations_hash,
        "sender" => full_validatios_sender,
    ]
    .unwrap()
    .lazy();

    let proposals = df![
        "seq" => p_seq,
        "prev" => p_prev,
        "consensus" => p_hash,
        "sender" => p_sender,
        "from" => propo_sender,
        "to" => propo_recv,
    ]
    .unwrap()
    .lazy();

    let validations = df![
        "sequence" => seqs,
        "hash" => hashs,
        "sender" => owner,
        "consensus" => consensus,
        "from" => val_sender,
        "to" => val_recv,
    ]
    .unwrap()
    .lazy();

    let proposals_new = proposals
        .filter(col("sender").eq(col("from")))
        .join(
            validations.clone(),
            [col("prev")],
            [col("hash")],
            JoinType::Left,
        )
        .groupby(["prev", "seq", "consensus", "sender"])
        .agg([
            col("from")
                .unique()
                .apply(|x| Ok(x.str_concat(",").into_series()), Default::default())
                .first(),
            col("to")
                .unique()
                .apply(
                    |x| {
                        if x.len() == 6 {
                            Ok(Series::from_vec("to", vec![0, 1, 2, 3, 4, 5, 6])
                                .str_concat(",")
                                .into_series())
                        } else {
                            Ok(x.str_concat(",").into_series())
                        }
                    },
                    Default::default(),
                )
                .first(),
            col("sequence").add(lit(1)).first(),
        ])
        .groupby(["sequence", "prev", "seq", "consensus", "to"])
        .agg([col("from")
            .unique()
            .sort(Default::default())
            .apply(|x| Ok(x.str_concat(",").into_series()), Default::default())
            .first()])
        .sort("seq", Default::default())
        .sort("sequence", Default::default())
        .fetch(50000)
        .unwrap()
        .select(["sequence", "prev", "seq", "consensus", "from", "to"])
        .unwrap();

    println!("{}", proposals_new);

    let mutations = proposals_new
        .clone()
        .lazy()
        .filter(col("to").eq(lit("0,1,2")));
    let res = proposals_new
        .clone()
        .lazy()
        .filter(col("from").neq(lit("3")))
        .join(
            mutations,
            [col("sequence")],
            [col("sequence")],
            JoinType::Inner,
        )
        .filter(
            col("from")
                .neq(lit("0,1,2"))
                .and(col("from").neq(lit("3")).and(col("from").neq(lit("4,5,6")))),
        )
        .fetch(50000)
        .unwrap()
        .select(["sequence", "seq", "consensus", "from", "to"])
        .unwrap();
    println!("{}", res);

    let mutations = proposals_new
        .clone()
        .lazy()
        .filter(col("to").eq(lit("0,1,2")));
    let res = proposals_new
        .lazy()
        .filter(col("from").neq(lit("3")))
        .join(
            mutations,
            [col("sequence")],
            [col("sequence")],
            JoinType::Inner,
        )
        .groupby([col("sequence")])
        .agg([col("seq").max()])
        .fetch(50000)
        .unwrap();
    println!("{}", res);

    let validations_new = validations
        .filter(col("sender").eq(col("from")))
        .join(
            full_validations.rename(["sender"], ["validated"]),
            [col("hash")],
            [col("hash")],
            JoinType::Outer,
        )
        .groupby(["hash", "sequence", "sender"])
        .agg([
            col("from")
                .unique()
                .apply(|x| Ok(x.str_concat(",").into_series()), Default::default())
                .first(),
            col("to")
                .unique()
                .apply(
                    |x| {
                        if x.len() == 6 {
                            Ok(Series::from_vec("to", vec![0, 1, 2, 3, 4, 5, 6])
                                .str_concat(",")
                                .into_series())
                        } else {
                            Ok(x.str_concat(",").into_series())
                        }
                    },
                    Default::default(),
                )
                .first(),
            col("validated")
                .unique()
                .apply(|x| Ok(x.str_concat(",").into_series()), Default::default())
                .first(),
        ])
        .groupby(["hash", "sequence", "to", "validated"])
        .agg([col("from")
            .unique()
            .sort(Default::default())
            .apply(|x| Ok(x.str_concat(",").into_series()), Default::default())
            .first()])
        .sort("from", Default::default())
        .sort("sequence", Default::default())
        .fetch(50000)
        .unwrap()
        .select(["sequence", "hash", "from", "to", "validated"])
        .unwrap();

    println!("{}", validations_new);
}
