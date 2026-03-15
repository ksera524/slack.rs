# slack.rs

Slack APIとS3互換ストレージ API へのラッパーを提供するRust製APIサーバーです。

OpenAPI definition: `openapi.yaml`

## API

- `GET /health`
  - body: なし
- `GET /openapi.json`
  - body: なし
- `POST /slack/message`
  - body: `{ "channel": "C123", "text": "hello" }`
- `POST /slack/upload/image?channel=C123&file_name=hello.png`
  - header: `Content-Type: image/png|image/jpeg|image/webp|image/gif`
  - body: 画像バイナリ
- `POST /slack/upload/pdf?channel=C123&file_name=document.pdf`
  - header: `Content-Type: application/pdf`
  - body: PDFバイナリ
- `POST /s3/put_object_base64`
  - body: `{ "bucket": "b", "key": "path/a.txt", "file_data_base64": "...", "content_type": "text/plain" }`
- `POST /s3/get_object_base64`
  - body: `{ "bucket": "b", "key": "path/a.txt" }`
- `POST /s3/head_object`
  - body: `{ "bucket": "b", "key": "path/a.txt" }`
- `POST /s3/delete_object`
  - body: `{ "bucket": "b", "key": "path/a.txt" }`
- `POST /s3/list_objects_v2`
  - body: `{ "bucket": "b", "prefix": "path/", "max_keys": 1000 }`
- `POST /s3/create_multipart_upload`
  - body: `{ "bucket": "b", "key": "large.bin" }`
- `POST /s3/upload_part_base64`
  - body: `{ "bucket": "b", "key": "large.bin", "upload_id": "...", "part_number": 1, "part_data_base64": "..." }`
- `POST /s3/complete_multipart_upload`
  - body: `{ "bucket": "b", "key": "large.bin", "upload_id": "...", "parts": [{ "part_number": 1, "e_tag": "\"...\"" }] }`
- `POST /s3/abort_multipart_upload`
  - body: `{ "bucket": "b", "key": "large.bin", "upload_id": "..." }`
- `POST /s3/list_parts`
  - body: `{ "bucket": "b", "key": "large.bin", "upload_id": "..." }`
- `POST /s3/list_multipart_uploads`
  - body: `{ "bucket": "b" }`
- `POST /s3/presigned_get_object`
  - body: `{ "bucket": "b", "key": "path/a.txt", "expires_in_secs": 900 }`
- `POST /s3/presigned_put_object`
  - body: `{ "bucket": "b", "key": "path/a.txt", "expires_in_secs": 900 }`
- `POST /s3/list_buckets`
  - body: なし
- `POST /s3/create_bucket`
  - body: `{ "bucket": "b" }`
- `POST /s3/head_bucket`
  - body: `{ "bucket": "b" }`
- `POST /s3/delete_bucket`
  - body: `{ "bucket": "b" }`

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
- `S3_ACCESS_KEY_ID` (必須)
- `S3_SECRET_ACCESS_KEY` (必須)
- `S3_REGION` (任意, デフォルト: `us-east-1`)
- `S3_ENDPOINT` (任意, 例: `http://rustfs.example.local:9000`)
- `S3_USE_PATH_STYLE` (任意, デフォルト: `true`)
- `S3_IGNORE_CERT_CHECK` (任意, デフォルト: `false`)
- `S3_SESSION_TOKEN` (任意)

## 起動

```bash
cargo run --bin slack
```

## APIテスト (tanu-rs)

MockのSlack APIを起動してテストします。外部Slackへの通信は行いません。

`api-tests` feature を使う場合、ローカル環境では OpenSSL 開発パッケージが必要です。

```bash
sudo apt-get update
sudo apt-get install -y pkg-config libssl-dev
```

Nix 環境では以下のように OpenSSL の pkg-config / 実行時ライブラリパスを指定して実行します。

```bash
PKG_CONFIG_PATH="/nix/store/<openssl-dev>/lib/pkgconfig" \
LD_LIBRARY_PATH="/nix/store/<openssl>/lib" \
cargo run --bin api_tests --features api-tests -- test
```

```bash
cargo run --bin api_tests --features api-tests -- test
```

## CI

GitHub Actionsで `cargo run --bin api_tests --features api-tests -- test` を実行します。
