# slack.rs

Slack APIへの投稿・ファイルアップロードを提供するRust製APIサーバーです。

## API

- `GET /health`
  - body: なし
- `POST /slack/message`
  - body: `{ "channel": "C123", "text": "hello" }`
- `POST /slack/upload/image?channel=C123&file_name=hello.png`
  - header: `Content-Type: image/png|image/jpeg|image/webp|image/gif`
  - body: 画像バイナリ
- `POST /slack/upload/pdf?channel=C123&file_name=document.pdf`
  - header: `Content-Type: application/pdf`
  - body: PDFバイナリ

## Error response (RFC9457)

エラーレスポンスは `application/problem+json` の最小セットで返します。

```json
{
  "type": "about:blank",
  "title": "Bad Request",
  "status": 400,
  "detail": "Body is not a valid PDF document"
}
```

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
cargo run --bin api_tests --features api-tests -- test
```

## CI

GitHub Actionsで `cargo run --bin api_tests --features api-tests -- test` を実行します。
