# Group Activity Bot

Group Activity Bot (GAB) is a Telgram bot that will keep track of user activities in groups, e.g.

<img width="1155" alt="Screenshot" src="https://user-images.githubusercontent.com/16304728/153413001-c55f3f46-e0f1-4661-9591-a9e1ed505892.png">

**GAB only works in group and must be a group admin in order to receive each user message in a group**

## Environment Variables

The bot can be configured using the following environment variables:

### Bot Configuration
- `TELOXIDE_TOKEN`: Your Telegram bot token (required)
- `DB_PATH`: Custom database file path (default: `.db/group.db`)
- `RUST_LOG`: Logging level (default: `info` in release, `debug` in debug mode)

### OpenAI Configuration
- `OPENAI_API_KEY`: Your OpenAI API key (required for AI features)
- `OPENAI_MODEL`: The model to use (default: `gpt-3.5-turbo`)
- `OPENAI_API_BASE`: Custom base URL for OpenAI API (default: `https://api.openai.com`)

## Features
|Command|Action|
|:-|:-|
|`/groupstats`|Returns a message containing the total number of messages exchanged in the group and the percentege of messages per user, with nice emojis for the top 3 users|
|`/userstats <username>`|Returns a message containing the percentage of messages of a specific user in a group|
|`/statsfile`|Returns a .csv file containing `(message timestamp, username)` for all the messages exchanged in a group, this is helpful to plot graphs and other analytics with other softwares|

- Track and store messages in a group
- Generate statistics about user participation
- AI responses when mentioned or randomly (with configurable chance)
- Support for custom OpenAI-compatible endpoints

### Run
```bash
docker compose up -d
```

## Running the Bot

```bash
export TELOXIDE_TOKEN=your_bot_token
export OPENAI_API_KEY=your_openai_key
# Optional: Use a different OpenAI-compatible API endpoint
export OPENAI_API_BASE=https://your-custom-endpoint.com
cargo run
```