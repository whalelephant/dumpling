# dumpling

## What is dumpling?

***dumpling*** is a command line interface tool designed for polkadot network validators. In the current PoA phrase of Polkadot, ***dumpling*** provides useful information for validators on the state of the network. 
***dumpling*** current has 3 subcommands.

`dumpling pulse` - for time dependent useful state information
```
dumpling-pulse
useful state information such as session index

USAGE:
    dumpling pulse [FLAGS]

FLAGS:
    -a, --activeEra       Active Era
    -b, --block           Current finalised block
    -h, --help            Prints help information
    -c, --plannedEra      Current Era (planned)
    -s, --sessionIndex    Current session index
    -V, --version         Prints version information

```

`dumpling validators` - for current validator lists
```
dumpling-validators
lists of validators and their information

USAGE:
    dumpling validators [FLAGS] [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -q, --queued     Queued validators with nominators' exposures and own exposure and ledger
    -s, --session    Session validators
    -V, --version    Prints version information
    -w, --waiting    waiting validators with their stake ledger and preferences

OPTIONS:
    -a, --account <accountId>    get waiting validator by accountId 

```

`dumpling nominators` - for current nominator list
```
dumpling-nominators
list of nominators and their information

USAGE:
    dumpling nominators [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --account <accountId>    get nominator by accountId 
```

#### Example:
`dumpling validators --waiting --account=AccountId

//TODO image

___
## Usage

#### CLI
1. Clone this repo
1. Cargo build --release
1. `./target/release/dumpling --help` to begin

#### Lib
As specified in `Cargo.toml`, you can also use this project as a library for your project in rust or other native modules.

#### Limitations

Due to the limited time for this challenge, currently this project works with a *_forked_* version of [substrate-api-client](https://github.com/scs/substrate-api-client). Changes made in this fork are:
- added `get_key_prefix` method and `get_keys` in the rpc client
- removed requirement for default value for `get_storage_map` *(hack!)*
- todo: update get_constants

***dumpling*** will collaborate with ***substrate-api-client*** to get the best solution for both projects.

___

#### Brief for challenge 2
Command-line Polkadot API interaction: 

- JS API interaction: Provide an easy interaction to write JavaScript code which interacts with a node. 

- Command-line Polkadot API interaction: Create a way to send different extrinsics from the command line to do different tasks – like seeing the current block height, list of validators, extrinsics in last block, etc. 

- Tools to help validators: Validators will be signaling their intention to nominate in the initial PoA phase. It would be great to have tools made specifically for them, such as seeing how much stake is behind others who have signaled their intention to validate.

TODO:
- move client() out of lib 
- flag required 
- active Era get time
- parse int sp_arithmatic


