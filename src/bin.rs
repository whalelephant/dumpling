use chrono::{TimeZone, Utc};
use clap::{load_yaml, App, Error, ErrorKind};
use comfy_table::*;
use dumpling::{ApiFilling, Nominator, Ss58Codec, WaitingValidator};
use rustyline::{error::ReadlineError, Editor};
use shellwords::split;
use std::collections::HashMap;

pub fn main() {
    let d = ApiFilling::new("127.0.0.1:9944", "polkadot");
    let yaml = load_yaml!("cli.yml");

    let mut wrapper = Editor::<()>::new();
    println!("Dumpling");
    loop {
        let readline = wrapper.readline("🥟 >> ");
        match readline {
            Ok(line) => {
                wrapper.add_history_entry(line.as_str());
                let m = split(&line).unwrap();
                let matches = App::from(yaml).get_matches_from_safe(m);

                match matches {
                    Ok(result) => match result.subcommand() {
                        ("pulse", Some(p_matches)) => {
                            if p_matches.is_present("activeEra") {
                                let mut t = Table::new();
                                table_header(&mut t, vec!["Active Era Index", "Started at"], 80);
                                let a = d.active_era(None);
                                match a {
                                    Some(info) => {
                                        let time = match info.start {
                                            Some(ts) => {
                                                (Utc.timestamp((ts / 1000) as i64, 0)).to_rfc2822()
                                            }
                                            None => String::from("Era has not started yet"),
                                        };
                                        add_row(
                                            &mut t,
                                            vec![
                                                (format!("{}", info.index), Color::Blue),
                                                (time, Color::Blue),
                                            ],
                                        );
                                        println!("{}", t);
                                    }
                                    None => println!("Active era information not available"),
                                }
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
                                    vec![(
                                        format!("{}", d.planned_era(None).unwrap()),
                                        Color::Blue,
                                    )],
                                );

                                println!("{}", t);
                            } else if p_matches.is_present("sessionIndex") {
                                let mut t = Table::new();
                                table_header(&mut t, vec!["Session Index"], 80);
                                add_row(
                                    &mut t,
                                    vec![(
                                        format!("{}", d.session_index(None).unwrap()),
                                        Color::Yellow,
                                    )],
                                );

                                println!("{}", t);
                            } else {
                                println!(
                                    "{:?}",
                                    Error {
                                        message:
                                            "Missing / Incorrect Arg; try --help for information"
                                                .to_string(),
                                        kind: ErrorKind::MissingArgumentOrSubcommand,
                                        info: None,
                                    }
                                );
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
                                        "Own Staked",
                                        "Claimed",
                                        "Nominators",
                                        "Commission",
                                    ],
                                    160,
                                );
                                let m = d.waiting_validators(None);

                                let mut t_total = Table::new();
                                table_header(&mut t_total, vec!["Total Waiting Validators"], 80);
                                add_row(&mut t_total, vec![(m.keys().len().to_string(), Color::Yellow)]);
                                println!("{}", t_total);

                                match v_matches.value_of("accountId") {
                                    Some(id) => display_validators(&mut t, &m, &id),
                                    None => {
                                        for i in m.keys() {
                                            display_validators(&mut t, &m, i)
                                        }
                                    }
                                }

                                println!("{}", t);
                            } else {
                                println!("Missing / Incorrect Arg; try --help for information");
                            }
                        }
                        ("nominators", Some(n_matches)) => {
                            let mut t = Table::new();
                            table_header(
                                &mut t,
                                vec![
                                    "Nominator Stash",
                                    "Staked",
                                    "Nominated Validators",
                                    "Era Submitted",
                                    "Suppressed",
                                ],
                                160,
                            );
                            let m = d.nominators(None);
                            let mut t_total = Table::new();
                            table_header(&mut t_total, vec!["Total Nominators"], 80);
                            add_row(&mut t_total, vec![(m.keys().len().to_string(), Color::Yellow)]);
                            println!("{}", t_total);

                            match n_matches.value_of("accountId") {
                                Some(id) => display_nominators(&mut t, &m, &id),
                                None => {
                                    for i in m.keys() {
                                        display_nominators(&mut t, &m, i)
                                    }
                                }
                            };
                            println!("{}", t);
                        }
                        ("exit", Some(_)) => {
                            println!("Bye!");
                            break;
                        }
                        ("", None) => println!("No subcommand was used"),
                        _ => unreachable!(),
                    },
                    Err(e) => {
                        println!("{}", e);
                        continue;
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("Bye!");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Bye!");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
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

fn display_nominators(t: &mut Table, m: &HashMap<String, Option<Nominator>>, i: &str) {
    match m.get(i) {
        Some(nominator) => {
            if let Some(n) = nominator {
                let mut targets: Vec<String> = Vec::new();
                for a in &(n.nominations).targets {
                    targets.push(a.to_ss58check());
                }
                add_row(
                    t,
                    vec![
                        (i.to_string(), Color::Blue),
                        (format!("{:#?}", n.staked), Color::Yellow),
                        (format!("{:?}", targets), Color::Red),
                        (format!("{}", n.nominations.submitted_in), Color::Yellow),
                        (format!("{:#?}", n.nominations.suppressed), Color::Magenta),
                    ],
                );
            }
        }
        None => {
            println!("{} is not on current nominators list", i);
        }
    }
}

fn display_validators(t: &mut Table, m: &HashMap<String, WaitingValidator>, i: &str) {
    match m.get(i) {
        Some(a) => {
            let mut row = vec![
                (i.to_string(), Color::Blue),
                (a.staked.to_string(), Color::Green),
                ("---".to_string(), Color::Yellow),
                (format!("{:?}", a.nominators), Color::Yellow),
                (
                    format!("{:?}", a.prefs.commission),
                    Color::Magenta,
                ),
            ];
            if let Some(l) = &a.ledger {
                let n = [(format!("{:#?}", l.claimed_rewards), Color::Green)];
                row.splice(2..3, n.iter().cloned());
            }

            add_row(t, row);
        }
        None => println!("{} is not on the waiting validators list", i),
    }
}
