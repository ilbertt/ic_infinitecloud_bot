# infinitecloud_bot

As Telegram offers unlimited cloud storage, it lacks of the main characteristics of the cloud services: folders structure, file names, etc.

Here comes the [Infinite Cloud Bot](https://t.me/infinitecloud_bot)! This bot simply keeps track of your files in a "filesystem", _without saving anything outside of the Telegram chat_ and so preserving your privacy.

The bot's backend runs in a [canister](https://dashboard.internetcomputer.org/canister/4vou4-lyaaa-aaaao-a3p4a-cai) on the [Internet Computer](https://internetcomputer.org/).

The previous version of the bot was developed in Node.js and is available at [ilbertt/infinitecloud_bot](https://github.com/ilbertt/infinitecloud_bot). It was always a pain to keep the bot server running _without having to care about the backend_, and this is the reason why we have chosen to use the [Internet Computer](https://internetcomputer.org/) for this new version of the bot.

## Usage

### Use the bot

Just start a chat with [@infinitecloud_bot](https://t.me/infinitecloud_bot) on Telegram! You'll find more usage instructions at the `/help` command in the bot chat.

### Deploy your own bot

#### 1. Requirements

Make sure you are familiar with the [Internet Computer](https://internetcomputer.org), its concepts and tools.

Make sure you have created a Telegram Bot ([instructions](https://core.telegram.org/bots#3-how-do-i-create-a-bot)) and have obtained a Bot API token from [@BotFather](https://t.me/BotFather) (we need this token in the next steps).

Create the following commands for your bot using [@BotFather](https://t.me/BotFather):

- `/help`
- `/info`
- `/mkdir`
- `/explorer`
- `/rename_file`
- `/move_file`

After creating the bot and its commands, create a random alphanumeric string of 256 characters max and add it to the `.env` file in the root directory under the `TELEGRAM_SECRET_TOKEN`. You can create the `.env` file by copying the [`.env.example`](./.env.example) file and renaming it to `.env`. This key will be used to authenticate requests coming from the Telegram servers. We need it in the next steps.

Make sure you have these tools installed:

- [Rust](https://www.rust-lang.org/tools/install)
- [dfx](https://internetcomputer.org/docs/current/developer-docs/setup/install)

#### 2. Deploy on the Internet Computer

Make sure you have obtained some cycles to cover the cost of the deployment. See [this guide](https://internetcomputer.org/docs/current/developer-docs/getting-started/deploy-and-manage) for more info.

```bash
# You need to remove the `canister_ids.json` file before deploying (ONLY THE FIRST TIME)
rm canister_ids.json
dfx deploy --ic
```

A new `canister_ids.json` file will be created in the root directory when the deployment is completed. You'll find the canister ID for the backend canister in the `backend.ic` field. We need it in the next step.

If you want to run the canister's bot locally, see the [Running the project locally](#running-the-project-locally) section.

#### 3. Configure Telegram to send messages to the bot via webhooks

Last step is to configure Telegram to invoke the bot via [webhooks](https://core.telegram.org/bots/api#setwebhook).

Make an HTTP POST request to the following URL:

```bash
curl -X POST https://api.telegram.org/bot<bot-token-from-botfather>/setWebhook?url=https://<backend-canister-id>.raw.icp0.io/&drop_pending_updates=True&secret_token=<TELEGRAM_SECRET_TOKEN>
```

### Running the project locally

If you want to test your project locally, you can use the following commands:

```bash
# Starts the replica, running in the background
dfx start

# Deploys your canisters to the replica and generates your candid interface
dfx deploy
```

In order to send messages to the bot by running the backend locally, you can deploy the canister using the following command:

```bash
./scripts/deploy-local-with-ngork-tunnel.sh
```

> Make sure you have [ngrok](https://ngrok.com/) installed.

You still need to configure Telegram to send messages to the bot via webhooks as described in [the previous step](#3-configure-telegram-to-send-messages-to-the-bot-via-webhooks).

## Testing

Unit tests are available with the following command:

```bash
./scripts/test.sh
```

## Linting

Linting (with [clippy](https://doc.rust-lang.org/stable/clippy)) is available with the following command:

```bash
./scripts/lint.sh
```

## Roadmap

- [ ] add support for `/delete_file` and `/delete_dir` commands
- [ ] add a [Telegram Mini App](https://core.telegram.org/bots/webapps) for the bot

## Contributing

Issues and Pull Requests are welcome!
