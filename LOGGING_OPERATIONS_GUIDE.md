# ログシステム運用ガイド

## 概要
`tracing`クレートをベースとした構造化ログシステムを実装しました。運用環境での可視性と問題診断を大幅に改善します。

## 主な機能

### 1. JSONL形式の構造化ログ
すべてのログはJSONL (JSON Lines) 形式で出力され、以下の情報を含みます：
- リクエストID（トレーサビリティ）
- 実行時間（パフォーマンス分析）
- エラー詳細（問題診断）
- コンテキスト情報（channel, file_name等）

1行に1つのJSONオブジェクトでストリーミング処理やログ解析ツールでの処理が容易です。

### 2. JSONLログ出力

JSONL (JSON Lines) 形式で固定されており、設定変更は不要です。
```bash
cargo run
```
```jsonl
{"timestamp":"2024-01-01T12:00:00.123","level":"INFO","message":"Successfully posted message to Slack","channel":"general","duration_ms":125,"request_id":"550e8400-e29b-41d4-a716-446655440000","target":"slack::handlers::slack_handler"}
{"timestamp":"2024-01-01T12:00:01.456","level":"DEBUG","message":"Slack API call successful","channel":"general","target":"slack::service::slack_service"}
```

各行が独立したJSONオブジェクトで、ストリーミング処理やログ解析ツールでの処理が容易です。


### 3. ログレベル設定

#### 基本設定
```bash
# 全体のログレベル
RUST_LOG=info cargo run

# 詳細なデバッグ
RUST_LOG=debug cargo run

# エラーのみ
RUST_LOG=error cargo run
```

#### モジュール別設定
```bash
# 特定モジュールのみデバッグ
RUST_LOG=info,slack::service=debug cargo run

# HTTPリクエストの詳細トレース
RUST_LOG=info,tower_http=trace cargo run

# Slack APIとハンドラーのデバッグ
RUST_LOG=info,slack::handlers=debug,slack::service=debug cargo run
```

### 4. リクエストトレーシング

すべてのHTTPリクエストに自動的にリクエストIDが付与されます：

```bash
curl -H "x-request-id: my-custom-id" http://localhost:3000/slack/message
```

レスポンスヘッダーにも同じIDが含まれます：
```
x-request-id: my-custom-id
```

### 5. パフォーマンス計測

各操作の実行時間が自動的に記録されます：
- HTTPリクエスト全体の処理時間
- Slack API呼び出しの実行時間
- ファイルアップロードの処理時間

### 6. エラートラッキング

エラーは適切なレベルで記録されます：
- `ERROR`: サーバーエラー、API呼び出し失敗
- `WARN`: クライアントエラー、Base64デコード失敗
- `INFO`: 正常な処理完了
- `DEBUG`: 詳細な処理情報

## 運用環境での設定例

### 本番環境
```bash
# .env
RUST_LOG=info,slack::service=warn
LOG_TARGET=true
LOG_LINE=false
```

### ステージング環境
```bash
# .env
RUST_LOG=info,slack=debug
LOG_TARGET=true
LOG_LINE=true
```

### 開発環境
```bash
# .env
RUST_LOG=debug
LOG_TARGET=true
LOG_THREAD=true
LOG_LINE=true
```

## ログ分析例

### エラーの検索
```bash
# JSONL形式のログからエラーを抽出
cargo run | jq 'select(.level == "ERROR")'

# jqを使ったリアルタイムフィルタリング
cargo run | jq -c 'select(.level == "ERROR")'
```

### 特定チャンネルのログ
```bash
# generalチャンネルへの投稿を抽出
cargo run | jq 'select(.channel == "general")'

# ストリーミング処理でリアルタイム監視
tail -f app.log | jq 'select(.channel == "general")'
```

### パフォーマンス分析
```bash
# 100ms以上かかったリクエストを抽出
cargo run | jq 'select(.duration_ms > 100)'

# 統計情報の算出
cargo run | jq -s 'map(select(.duration_ms)) | {avg: (map(.duration_ms) | add / length), max: (map(.duration_ms) | max)}'
```

### リクエストIDでの追跡
```bash
# 特定のリクエストIDのログを抽出
cargo run | jq 'select(.request_id == "550e8400-e29b-41d4-a716-446655440000")'

# grepで高速検索
cargo run | grep '"request_id":"550e8400-e29b-41d4-a716-446655440000"'
```

## トラブルシューティング

### ログが多すぎる場合
```bash
# 特定モジュールを無効化
RUST_LOG=info,hyper=error,reqwest=error cargo run
```

### 特定の処理をデバッグ
```bash
# Slack APIのみ詳細ログ
RUST_LOG=error,slack::service=trace cargo run
```

### パフォーマンス問題の調査
```bash
# すべてのタイミング情報を表示
RUST_LOG=debug cargo run | jq 'select(.duration_ms != null)'

# 遅いリクエストのリアルタイム監視
cargo run | jq -c 'select(.duration_ms > 500) | {time: .timestamp, endpoint: .target, duration: .duration_ms}'
```

## ベストプラクティス

1. **JSONL形式で固定されたログ出力**
   - ログ集約ツール（ELK Stack、Fluentd、Datadog等）との連携が容易
   - ストリーミング処理に最適化されている
   - 設定変更の必要がなく、運用が簡素

2. **適切なログレベルの設定**
   - 本番: `info`
   - ステージング: `info,slack=debug`
   - 開発: `debug`

3. **リクエストIDの活用**
   - フロントエンドから送信されたIDでエンドツーエンドのトレース

4. **定期的なログローテーション**
   - systemdやlogrotateと組み合わせて使用

5. **機密情報の除外**
   - トークンやパスワードは絶対にログに含めない

## 監視アラートの設定例

### エラー率の監視
```
ERROR レベルのログが1分間に10件以上発生したらアラート
```

### レスポンスタイムの監視
```
duration_ms > 1000 のログが5分間に5件以上発生したらアラート
```

### 特定エラーの監視
```
"Failed to post message to Slack" が発生したら即座にアラート
```