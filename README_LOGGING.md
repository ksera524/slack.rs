# JSONL構造化ログシステム

## 概要

このアプリケーションは、JSONL（JSON Lines）形式で統一された構造化ログシステムを採用しています。すべてのログは1行に1つのJSONオブジェクトとして出力され、運用環境での解析・監視に最適化されています。
## ログ形式

### JSONL形式（固定）
```jsonl
{"timestamp":"2025-09-16 03:40:36.770","level":"INFO","message":"Starting application","service":"slack-rs","version":"0.1.0"}
{"timestamp":"2025-09-16 03:40:36.770","level":"INFO","message":"Configuration loaded successfully","config_loaded":true}
{"timestamp":"2025-09-16 03:40:36.800","level":"INFO","message":"Starting HTTP server","addr":"0.0.0.0:3000","port":3000}
```

各行が独立したJSONオブジェクトのため、以下の利点があります：
- ストリーミング処理での行単位解析
- 並列処理による高速化
- 部分的なファイル破損への耐性
- メモリ効率の向上

## 設定

### 環境変数
```bash
# ログレベル設定
RUST_LOG=info                    # 基本設定
RUST_LOG=debug                   # 詳細ログ
RUST_LOG=info,slack::service=debug  # モジュール別設定

# ログ詳細設定
LOG_TARGET=true                  # モジュール名表示
LOG_THREAD=true                  # スレッド情報表示
LOG_LINE=true                    # ファイル・行番号表示
```

### 設定例

#### 本番環境
```bash
RUST_LOG=info,slack::service=warn
LOG_TARGET=true
LOG_LINE=false
```

#### 開発環境
```bash
RUST_LOG=debug
LOG_TARGET=true
LOG_THREAD=true
LOG_LINE=true
```

## ログ解析

### 基本的な解析
```bash
# エラーログの抽出
cargo run | jq 'select(.level == "ERROR")'

# 特定チャンネルのログ
cargo run | jq 'select(.channel == "general")'

# パフォーマンス分析
cargo run | jq 'select(.duration_ms > 100)'

# リクエストID追跡
cargo run | jq 'select(.request_id == "uuid-here")'
```

### 高速検索（grep使用）
```bash
# 文字列での高速検索
cargo run | grep '"level":"ERROR"'
cargo run | grep '"channel":"general"'
cargo run | grep '"request_id":"uuid-here"'
```

### 統計分析
```bash
# 平均レスポンス時間
cargo run | jq -s 'map(select(.duration_ms)) | {avg: (map(.duration_ms) | add / length)}'

# エラー数のカウント
cargo run | jq -s 'map(select(.level == "ERROR")) | length'
```

## 運用での活用

### ログ集約システム
- **ELK Stack**: Logstashで行単位処理
- **Fluentd**: ストリーミング取り込み
- **Datadog**: 自動JSON解析
- **Prometheus**: メトリクス抽出

### リアルタイム監視
```bash
# エラー監視
tail -f app.log | jq 'select(.level == "ERROR")'

# 遅延監視
tail -f app.log | jq 'select(.duration_ms > 1000)'

# チャンネル別監視
tail -f app.log | jq 'select(.channel)'
```

### ログローテーション
```bash
# logrotate設定例
/var/log/slack-rs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    missingok
    notifempty
}
```

## トレーシング情報

### 自動付与される情報
- `timestamp`: ローカル時刻（ミリ秒精度）
- `level`: ログレベル（ERROR, WARN, INFO, DEBUG, TRACE）
- `message`: ログメッセージ
- `target`: モジュール名（LOG_TARGET=true時）

### HTTPリクエスト情報
- `request_id`: 一意のリクエストID
- `method`: HTTPメソッド
- `path`: リクエストパス
- `status`: レスポンスステータス
- `duration_ms`: 処理時間（ミリ秒）

### Slack API情報
- `channel`: Slackチャンネル名
- `file_name`: アップロードファイル名
- `file_size`: ファイルサイズ
- `api_endpoint`: 呼び出しAPIエンドポイント

## パフォーマンス最適化

### ログレベルの調整
```bash
# 本番環境（必要最小限）
RUST_LOG=warn

# 監視強化時
RUST_LOG=info,slack=debug

# 問題調査時
RUST_LOG=debug
```

### 処理効率
- 行単位処理でメモリ使用量一定
- 並列処理でCPU効率最大化
- インデックス不要な高速grep検索

## アラート設定例

### エラー率監視
```bash
# 1分間にERRORが5件以上でアラート
tail -f app.log | jq -r 'select(.level == "ERROR") | .timestamp' | \
  awk '{count++} END {if(count >= 5) print "ALERT: High error rate"}'
```

### レスポンス時間監視
```bash
# 1秒以上のリクエストでアラート
tail -f app.log | jq 'select(.duration_ms > 1000)' | \
  jq -r '"ALERT: Slow request: " + .path + " (" + (.duration_ms|tostring) + "ms)"'
```

## ベストプラクティス

1. **ログレベルの適切な使用**
   - ERROR: 即座に対応が必要なエラー
   - WARN: 注意が必要な状況
   - INFO: 重要なイベント
   - DEBUG: 詳細な処理情報

2. **構造化フィールドの活用**
   - 検索しやすいフィールド名
   - 一貫した命名規則
   - 適切なデータ型

3. **機密情報の除外**
   - トークンやパスワードは絶対に記録しない
   - 個人情報の適切なマスキング

4. **パフォーマンス考慮**
   - 本番環境では適切なログレベル設定
   - 大量ログ出力時のディスク容量監視

この統一されたJSONL形式により、運用環境での可視性・問題診断能力・自動化対応が大幅に向上します。
