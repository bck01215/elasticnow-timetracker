# ElasticNow CLI
This project was inspired by [Preston Gibbs](mailto:pgibbs1@liberty.edu) and his hate for time tracking.

## Usage
### Authentication
To initialize the repo you will need to run `elasticnow setup`. This will generate a config.toml. The location is dependent on the operating system but can be found at the top of the help output.

| Flag | Description |
| --- | --- |
| `--id <ID>` | The ElasticNow ID (retrieved from ElasticNow instance) [env: ELASTICNOW_ID] |
| `--instance <INSTANCE>` | The ElasticNow instance [env: ELASTICNOW_INSTANCE=] |
| `--sn-instance <SN_INSTANCE>` | The ServiceNow Instance (e.g. libertydev, liberty) [env: SN_INSTANCE=] |
| `--sn-username <SN_USERNAME>` | The ServiceNow Username [env: SN_USERNAME=] |
| `--sn-password <SN_PASSWORD>` | The ServiceNow Password [env: SN_PASSWORD] |
| `-b, --bin <BIN>` | Override default bin for searching (defaults to user's assigned bin) |
| `-h, --help` | Print help |

Usage: `elasticnow setup [OPTIONS] --id <ID> --instance <INSTANCE> --sn-instance <SN_INSTANCE> --sn-username <SN_USERNAME> --sn-password <SN_PASSWORD>`

### Time Tracking
Time tracking is dependent on the initial setup. You can use the search flag to search for an existing ticket in your bin (override with --bin), or create a new ticket.

When searching, the CLI will return a list for the user to choose from after querying all active tickets in the bin matching the search key words.

| Flag | Description |
| --- | --- |
| `-n, --new` | Creates a new ticket instead of updating an existing one ( cannot be used with `--search` ) |
| `-c, --comment <COMMENT>` | Comment for time tracking |
| `--time-worked <TIME_WORKED>` | Add time in the format of 1h1m where 1 can be replaced with any number (hours must be less than 24) |
| `-s, --search <SEARCH>` | Keyword search using ElasticNow (returns all tickets in bin by default) |
| `-b, --bin <BIN>` | Override default bin for searching (defaults to user's assigned bin or override in config.toml) |
| `-h, --help` | Print help |

Usage: `elasticnow timetrack [OPTIONS] --comment <COMMENT> --time-worked <TIME_WORKED> --search <SEARCH>`
