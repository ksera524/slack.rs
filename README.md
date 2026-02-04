# slack.rs

Slack APIへの投稿・ファイルアップロードを提供するRust製APIサーバーです。

## API

- `GET /health`
  - body: なし
- `POST /slack/message`
  - body: `{ "channel": "C123", "text": "hello" }`
- `POST /slack/upload_base64`
  - body: `{ "file_name": "hello.txt", "file_data_base64": "...", "channel": "C123" }`

## 環境変数

- `SLACK_BOT_TOKEN` (必須)
- `SLACK_API_BASE_URL` (任意, デフォルト: `https://slack.com/api`)

## 起動

```bash
cargo run --bin slack
```

## APIテスト (tanu-rs)

MockのSlack APIを起動してテストします。外部Slackへの通信は行いません。

```bash
cargo run --bin api_tests -- test
```

## CI

GitHub Actionsで `cargo run --bin api_tests -- test` を実行します。
