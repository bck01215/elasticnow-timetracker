# ElasticNow CLI

This project simplifies timetracking in servicenow for those who find it difficult to navigate.

## Usage

### Autocomplete

`elasticnow --generate   [possible values: bash, elvish, fish, powershell, zsh]` outputs the autocompletion that can be pushed to proper file for autocomplete.

### Authentication

To initialize the repo you will need to run `elasticnow setup`. This will generate a config.toml. The location is dependent on the operating system but can be found at the top of the help output.

| Flag                          | Description                                                                 |
| ----------------------------- | --------------------------------------------------------------------------- |
| `--id <ID>`                   | The ElasticNow ID (retrieved from ElasticNow instance) [env: ELASTICNOW_ID] |
| `--instance <INSTANCE>`       | The ElasticNow instance [env: ELASTICNOW_INSTANCE=]                         |
| `--sn-instance <SN_INSTANCE>` | The ServiceNow Instance (e.g. libertydev, liberty) [env: SN_INSTANCE=]      |
| `--sn-username <SN_USERNAME>` | The ServiceNow Username [env: SN_USERNAME=]                                 |
| `--sn-password <SN_PASSWORD>` | The ServiceNow Password [env: SN_PASSWORD]                                  |
| `-b, --bin <BIN>`             | Override default bin for searching (defaults to user's assigned bin)        |
| `-h, --help`                  | Print help                                                                  |

Usage: `elasticnow setup [OPTIONS] --id <ID> --instance <INSTANCE> --sn-instance <SN_INSTANCE> --sn-username <SN_USERNAME> --sn-password <SN_PASSWORD>`

### Time Tracking

Time tracking is dependent on the initial setup. You can use the search flag to search for an existing ticket in your bin (override with --bin), or create a new ticket.

When searching, the CLI will return a list for the user to choose from after querying all active tickets in the bin matching the search key words.

| Flag                          | Description                                                                                         |
| ----------------------------- | --------------------------------------------------------------------------------------------------- |
| `-n, --new`                   | Creates a new ticket instead of updating an existing one ( cannot be used with `--search` )         |
| `-a, --all`                   | Returns all item in the bin instead of searching                                                    |
| `-c, --comment <COMMENT>`     | Comment for time tracking                                                                           |
| `--time-worked <TIME_WORKED>` | Add time in the format of 1h1m where 1 can be replaced with any number (hours must be less than 24) |
| `-s, --search <SEARCH>`       | Keyword search using ElasticNow (returns all tickets in bin by default)                             |
| `-b, --bin <BIN>`             | Override default bin for searching (defaults to user's assigned bin or override in config.toml)     |
| `--no-tkt`                    | Uses timetracking without a ticket                                                                  |
| `-h, --help`                  | Print help                                                                                          |

Usage: `elasticnow timetrack [OPTIONS] --comment <COMMENT> --time-worked <TIME_WORKED> --search <SEARCH>`

### Standard Changes

This just uses the ServiceNow API to query STD CHG templates and prompt the user for correct one. Alternatively, provide the sys_id of the template to avoid being prompted.

Options:
| Flag | Description |
| --- | --- |
| `-s, --search <SEARCH>` | Search for a STD CHG template to create the CHG with |
| `-b, --bin <BIN>` | Override default assignment group when creating the CHG |
| `-t, --template-id <TEMPLATE_ID>` | Use a known template ID to skip the prompt |
| `-h, --help` | Print help |

Usage: `elasticnow std-chg [OPTIONS]`

### Report

This gets the user's current time tracking and returns the `--top` results and total time tracking for the range. The duration flags (`--since` and `--until`) default to the current work week. If total is below 32 hours it will return red

Options:
| Flag | Description |
| --- | --- |
| `-u, --user <USER>` | Override the default user in the report |
| `--since <SINCE>` | Start date of search (defaults to 2024-06-24) |
| `--until <UNTIL>` | End date of search (defaults to 2024-06-26) |
| `-t, --top <TOP>` | Limit the number of cost centers returned in the report. Any extra fields will be grouped into other [default: 10]|
| `-h, --help` | Print help |

Usage: `elasticnow report [OPTIONS]`
