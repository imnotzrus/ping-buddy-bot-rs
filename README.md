# Ping Buddy Bot 🦀

A Telegram bot that helps you organize and tag people in group chats using topic-based subscriptions. Written in Rust for performance and reliability.

## Features

- 📌 **Topic-based subscriptions**: Create custom topics and subscribe users
- 🏷️ **Smart tagging**: Ping all subscribers of a topic with a simple command
- 👥 **User management**: Easy subscription/unsubscription through inline buttons
- 🔒 **Input validation**: Comprehensive validation for topic names and user inputs
- 📊 **Logging**: Structured logging for monitoring and debugging
- ⚡ **Performance**: Built with Rust for speed and memory efficiency

## Commands

- `/list` - Show all available topics and your subscriptions
- `/all` - Tag everyone subscribed to the "general" topic
- `/topic_name` - Tag all subscribers of a specific topic

## Setup

### Prerequisites

- Rust 1.70 or higher
- A Telegram Bot Token (get one from [@BotFather](https://t.me/botfather))

### Environment Variables

Set up the following environment variables:

| Variable | Description | Required |
|----------|-------------|----------|
| `TELOXIDE_TOKEN` | Your Telegram bot token | ✅ Yes |
| `INBOUND` | Webhook inbound URL (e.g., `https://your-domain.com/webhook`) | ✅ Yes |
| `OUTBOUND` | Webhook outbound URL (e.g., `https://api.telegram.org`) | ✅ Yes |
| `RUST_LOG` | Log level (`debug`, `info`, `warn`, `error`) | ❌ No (default: `info`) |

### Build and Run

```bash
# Build release version
cargo build --release

# Run the bot
./target/release/ping-buddy
```

### Development

```bash
# Run in development mode with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check for errors
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Usage

### Creating Topics

1. Add the bot to your Telegram group
2. Use `/list` to see available topics
3. Click "Create New Topic" button
4. Reply to the bot's message with your topic name
5. Topic names must:
   - Start with a letter
   - Contain only alphanumeric characters and underscores
   - Be 1-50 characters long
   - Not be a reserved name (all, everyone, here, channel)

### Subscribing to Topics

1. Use `/list` to see all topics
2. Click on a topic to subscribe/unsubscribe
3. Topics with ✓ are your active subscriptions

### Tagging Users

- Use `/topic_name` to ping all subscribers
- Use `/all` to ping everyone in the general topic
- The bot will list all tagged users in the response

## Architecture

### Project Structure

```
src/
├── main.rs              # Application entry point
├── lib.rs               # Library root and bot setup
├── error.rs             # Custom error types
├── constants.rs         # Application constants
├── validation.rs        # Input validation logic
├── env.rs               # Environment configuration
├── storage.rs           # In-memory data storage
├── command.rs           # Bot commands definition
├── utils.rs             # Utility macros
└── handler/             # Message and callback handlers
    ├── mod.rs
    ├── callback.rs      # Inline button handlers
    ├── helper_messages.rs
    └── message/
        ├── mod.rs
        ├── static/      # Command handlers (/list, /all)
        └── dynamic/     # Topic ping and creation handlers
```

### Key Components

- **Storage**: Thread-safe in-memory storage using `RwLock` and `HashMap`
- **Error Handling**: Custom error types with `thiserror` for better error messages
- **Validation**: Comprehensive input validation for security
- **Logging**: Structured logging with different levels for debugging

## Configuration Limits

- Maximum topics per chat: 100
- Maximum subscribers per topic: 1000
- Maximum topic name length: 50 characters
- Bot message TTL: 30 seconds (for error messages)

## Security

- Input validation on all user-provided data
- URL validation for webhook configuration
- Authorization checks for callback queries
- Reserved topic names to prevent conflicts

## Contributing

Contributions are welcome! Please ensure:

1. Code passes `cargo test`
2. Code is formatted with `cargo fmt`
3. No warnings from `cargo clippy`
4. Add tests for new features

## License

See [LICENSE](LICENSE) file for details.

## Acknowledgments

Built with:
- [teloxide](https://github.com/teloxide/teloxide) - Telegram bot framework
- [tokio](https://tokio.rs/) - Async runtime
- [thiserror](https://github.com/dtolnay/thiserror) - Error handling
