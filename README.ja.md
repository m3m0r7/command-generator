# Command Generator

`command-generator` は、LLM を使ってシェルコマンドを対話的に生成する Rust 製 CLI ツールです。  
コマンド生成だけでなく、生成結果の妥当性検証、セッション保存・再開、不足情報の追加質問を行います。

## 特徴

- 対話型コマンド生成（デフォルト起動）
- OpenAI / Gemini / Claude をサポート
- Function Calling ベースの 3 ツール設計
  - `deliver_command`: 最終コマンドを返す
  - `ask_yes_no_question`: yes/no で答える確認質問
  - `ask_text_question`: 文字列など自由入力の確認質問
- 妥当性検証
  - シェル構文チェック（`$SHELL -n -c`）
  - コマンド解決チェック（`which` + `command -v`）
  - alias 衝突検出（`builtin` / `command` / `\` プレフィックス誘導）
  - プレースホルダ（`<STRING>` 等）拒否
  - 安全なコマンドのみ `/tmp` で実行スモークチェック
- セッション保存と再開（UUID）
- `--resume` 時に過去コンテキストを起動直後に表示
- `-c/--copy` で出力コマンドをクリップボードへコピー

## 必要環境

- Rust（stable）
- いずれかの API キー
  - OpenAI: `OPENAI_API_KEY`
  - Gemini: `GEMINI_API_KEY` または `GOOGLE_API_KEY`
  - Claude: `ANTHROPIC_API_KEY`

## インストール

```bash
cargo install --path .
```

任意で alias:

```bash
alias cg="command-generator"
```

## クイックスタート

```bash
cg
```

例:

```text
> $PATH に特定の文字列が入っていたら yes、それ以外は no と出力するようにして
? PATH に含まれるか確認したい文字列を入力してください: mytool
[[ ":$PATH:" == *":mytool:"* ]] && print -r -- yes || print -r -- no
> exit
Good Bye!
```

## 対話ログ例

### 例1: 不足値をテキスト質問で補完

```text
> $PATH に特定の文字列が入っていたら yes、それ以外は no と出力するようにして
? PATH に含まれるか確認したい文字列を入力してください: mytool
[[ ":$PATH:" == *":mytool:"* ]] && print -r -- yes || print -r -- no
```

### 例2: yes/no 質問で条件を確定

```text
> 再帰で *.rs を検索して
? 隠しディレクトリ（.git など）も検索対象に含めますか？ [y/n]: n
find . -type f -name '*.rs' -not -path '*/.*/*'
```

### 例3: resume 時に過去コンテキスト表示

```text
$ cg --resume e13d0964-7710-41e8-a7bc-f5d197b7c1f7
Resumed context (showing 1 turn(s) of 1):
> pwd を出力
pwd
---
Interactive mode. Type exit to finish.
```

## CLI オプション

```text
-m, --model <MODEL>                       モデル名 or provider:model
-k, --key <KEY>                           API キー（環境変数より優先）
    --show-models-list                    モデル一覧表示
-c, --copy                                生成コマンドをコピー
-r, --resume <UUID>                       セッション再開
    --once <REQUEST>                      非対話で 1 回だけ実行
    --history-lines <N>                   シェル履歴取り込み行数（default: 80）
    --generated-history-lines <N>         過去生成コマンド取り込み行数（default: 80）
    --context-turns <N>                   セッション文脈の最大ターン数（default: 12）
    --max-attempts <N>                    検証失敗時の再生成回数（default: 3）
```

## モデルとプロバイダ

### モデル指定

- `-m openai:gpt-5.2`
- `-m gemini:gemini-2.5-flash`
- `-m claude:claude-sonnet-4-5`
- `-m openai` のように provider のみ指定も可

### 既定プロバイダ

API キーの存在順で自動選択します:

1. `OPENAI_API_KEY`
2. `GEMINI_API_KEY` / `GOOGLE_API_KEY`
3. `ANTHROPIC_API_KEY`

### モデル一覧

```bash
cg --show-models-list
cg --show-models-list -m gemini
```

モデル一覧はキャッシュされ、TTL は 24 時間です。

## セッション保存・再開

各生成は UUID を持つセッションとして保存されます。

```bash
cg --resume <uuid>
# or
cg -r <uuid>
```

保存先（既定）:

- `~/.command-generator/sessions/*.json`
- `~/.command-generator/.cache/meta.json`

`COMMAND_GENERATOR_DIR` を設定すると保存先ルートを変更できます。

## 検証ポリシー

生成コマンドは以下で検証されます。

1. シェル構文チェック
2. 解決可能コマンドかどうか
3. alias 衝突（必要なら `builtin`, `command`, `\` を要求）
4. プレースホルダ禁止
5. 安全判定されたコマンドのみ実行スモークチェック

注: `--once` では対話質問に答えられないため、質問が必要な要求はエラーになります。  
その場合は対話モード（`cg`）を使用してください。

## 開発用コマンド

```bash
make build
make test
make fmt
make clippy
make release
make install
make command-generator-build
```

## ライセンス

`LICENSE` を参照してください。
