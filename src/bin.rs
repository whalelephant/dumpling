use clap::{App, load_yaml};
use comfy_table::*;
use dumpling::{AccountId, Dumpling, Ss58Codec};
use std::collections::HashMap;

pub fn main() {
    let d = Dumpling::new("127.0.0.1:9944", "polkadot");
    let yaml = load_yaml!("cli.yml");
    let matches = App::from(yaml).get_matches();

    match matches.subcommand() {
        ("pulse", Some(p_matches)) => {
            if p_matches.is_present("activeEra") {
                let mut t = Table::new();
                table_header(&mut t, vec!["Active Era"], 80);
                add_row(
                    &mut t,
                    vec![(format!("{:?}", d.active_era(None).unwrap()), Color::Blue)],
                );

                println!("{}", t);
            } else if p_matches.is_present("block") {
                let mut t = Table::new();
                table_header(
                    &mut t,
                    vec!["Finalised block hash", "Finalised block number"],
                    160,
                );
                let b = d.finalized_head();
                add_row(
                    &mut t,
                    vec![
                        (format!("{:?}", b.0.unwrap()), Color::Blue),
                        (format!("{}", b.1.unwrap().number), Color::Yellow),
                    ],
                );
                println!("{}", t);
            } else if p_matches.is_present("plannedEra") {
                let mut t = Table::new();
                table_header(&mut t, vec!["Planned Era"], 80);
                add_row(
                    &mut t,
                    vec![(format!("{}", d.planned_era(None).unwrap()), Color::Blue)],
                );

                println!("{}", t);
            } else if p_matches.is_present("sessionIndex") {
                let mut t = Table::new();
                table_header(&mut t, vec!["Session Index"], 80);
                add_row(
                    &mut t,
                    vec![(format!("{}", d.session_index(None).unwrap()), Color::Yellow)],
                );

                println!("{}", t);
            } else {
                // todo error / requirements
                println!("flag required")
            }
        }
        ("validators", Some(v_matches)) => {
            if v_matches.is_present("session") {
                let mut t = Table::new();
                table_header(&mut t, vec!["Seesion Validator Stash"], 80);
                let v = d.session_validators(None).unwrap();
                for i in v {
                    add_row(&mut t, vec![(i.to_ss58check(), Color::Yellow)]);
                }

                println!("{}", t);
            } else if v_matches.is_present("queued") {
                let mut t = Table::new();
                table_header(
                    &mut t,
                    vec![
                        "Queued Validator Stash",
                        "Total Exposure",
                        "Own",
                        "Others (Stash key: value)",
                    ],
                    160,
                );
                let v = d.queued_validators(None).unwrap();
                for i in v.exposures {
                    let mut fmt_exposures = HashMap::new();
                    let indv_exposures = (i.1).others;
                    for e in indv_exposures {
                        fmt_exposures.insert(e.who.to_ss58check(), e.value);
                    }
                    add_row(
                        &mut t,
                        vec![
                            (i.0.to_ss58check(), Color::Blue),
                            (format!("{}", (i.1).total), Color::Yellow),
                            (format!("{}", (i.1).own), Color::Yellow),
                            (format!("{:#?}", fmt_exposures), Color::Magenta),
                        ],
                    );
                }

                println!("{}", t);
            } else if v_matches.is_present("waiting") {
                let mut t = Table::new();
                table_header(
                    &mut t,
                    vec![
                        "Waiting Validator Stash",
                        "Total",
                        "Active",
                        "Unlocking",
                        "Claimed",
                        "Commission",
                    ],
                    160,
                );
                let m = d.waiting_validators(None);

                if let Some(account) = v_matches.value_of("accountId") {
                    let account_id = AccountId::from_ss58check(account)
                        .unwrap_or_else(|_| panic!("Error in getting SS58 account"));
                    match m.get(&account_id) {
                        Some(a) => {
                            let mut row = vec![
                                (account.to_string(), Color::Blue),
                                ("---".to_string(), Color::Yellow),
                                ("---".to_string(), Color::Yellow),
                                ("---".to_string(), Color::Yellow),
                                ("---".to_string(), Color::Yellow),
                                (
                                    format!("{:?}", a.1.as_ref().unwrap().commission),
                                    Color::Magenta,
                                ),
                            ];
                            if let Some(l) = &a.0 {
                                let n = [
                                    (format!("{}", l.total), Color::Yellow),
                                    (format!("{}", l.active), Color::Yellow),
                                    (format!("{:#?}", l.unlocking), Color::Yellow),
                                    (format!("{:#?}", l.claimed_rewards), Color::Yellow),
                                ];
                                row.splice(1..5, n.iter().cloned());
                            }
                            add_row(&mut t, row);
                        }
                        None => panic!("{} is not on the waiting validators list", account),
                    }
                } else {
                    for (k, v) in m.iter() {
                        let mut row = vec![
                            (k.to_ss58check(), Color::Blue),
                            ("---".to_string(), Color::Yellow),
                            ("---".to_string(), Color::Yellow),
                            ("---".to_string(), Color::Yellow),
                            ("---".to_string(), Color::Yellow),
                            (
                                format!("{:?}", v.1.as_ref().unwrap().commission),
                                Color::Magenta,
                            ),
                        ];
                        if let Some(l) = &v.0 {
                            let n = [
                                (format!("{}", l.total), Color::Yellow),
                                (format!("{}", l.active), Color::Yellow),
                                (format!("{:#?}", l.unlocking), Color::Yellow),
                                (format!("{:#?}", l.claimed_rewards), Color::Yellow),
                            ];
                            row.splice(1..5, n.iter().cloned());
                        }

                        add_row(&mut t, row);
                    }
                }
                println!("{}", t);
                println!("Total waiting validators: {}", m.keys().len());
            } else {
                // todo error / requirements
                println!("flag required")
            }
        }
        ("nominators", Some(n_matches)) => {
            let mut t = Table::new();
            table_header(
                &mut t,
                vec!["Nominator Stash", "Targets", "Era Submitted", "Suppressed"],
                160,
            );
            let m = d.nominators(None);

            if let Some(account) = n_matches.value_of("accountId") {
                match m.get(account) {
                    Some(n) => match n {
                        Some(l) => {
                            let mut targets: Vec<String> = Vec::new();
                            for a in &l.targets {
                                targets.push(a.to_ss58check());
                            }
                            add_row(
                                &mut t,
                                vec![
                                    (account.to_string(), Color::Blue),
                                    (format!("{:#?}", targets), Color::Yellow),
                                    (format!("{}", l.submitted_in), Color::Yellow),
                                    (format!("{:#?}", l.suppressed), Color::Magenta),
                                ],
                            );
                        }
                        None => panic!("Account has no Nominations"),
                    },
                    None => panic!("{} is not on current nominators list", account),
                }
            } else {
                for (k, v) in m.iter() {
                    match &v {
                        Some(l) => {
                            let mut targets: Vec<String> = Vec::new();
                            for a in &l.targets {
                                targets.push(a.to_ss58check());
                            }
                            add_row(
                                &mut t,
                                vec![
                                    (format!("{}", k), Color::Blue),
                                    (format!("{:#?}", targets), Color::Yellow),
                                    (format!("{}", l.submitted_in), Color::Yellow),
                                    (format!("{:#?}", l.suppressed), Color::Magenta),
                                ],
                            );
                        }
                        None => continue,
                    }
                }
            }
            println!("{}", t);
        }
        ("", None) => println!("No subcommand was used"),
        _ => unreachable!(),
    }
}

fn table_header(t: &mut Table, headers: Vec<&str>, width: u16) {
    t.set_content_arrangement(ContentArrangement::Dynamic)
        .set_table_width(width);
    let mut t_header = Vec::new();
    for h in headers {
        t_header.push(
            Cell::new(h)
                .add_attribute(Attribute::Bold)
                .bg(Color::Black)
                .fg(Color::Green),
        );
    }
    t.set_header(t_header);
}

fn add_row(t: &mut Table, rows: Vec<(String, Color)>) {
    let mut t_row = Vec::new();
    for r in rows {
        t_row.push(Cell::new(r.0).bg(Color::Black).fg(r.1));
    }
    t.add_row(t_row);
}
