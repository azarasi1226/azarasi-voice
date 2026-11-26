# azarasi-voice

Windows 用の音声入力ツール。ホットキーを押している間だけ録音し、ローカルの音声認識で文字にして、（必要なら）Claude で整えてから、開いているウィンドウに貼り付ける。

## 方針

ゲームと並行して使えるように、**待機中は軽く、話したときだけ重くなる**ことを最優先にする。

- GPU / VRAM は使わない。音声認識は CPU で動かす。
- 待機中はキー入力の監視だけ。モデルはホットキーを押した瞬間に読み込み始め（話している間に終わる）、しばらく使わなければメモリから外す。

## 仕様

| 項目 | 内容 |
|---|---|
| 音声認識 | Parakeet 日本語版（`parakeet-tdt_ctc-0.6b-ja` int8）を sherpa-onnx で CPU 実行 |
| ホットキー | Win + 左 Ctrl を押している間だけ録音 |
| 貼り付け | クリップボードに入れて Ctrl+V を送り、元のクリップボードの中身は戻す |
| 整形 | Claude Haiku（Anthropic API）。オフ / 軽め / しっかり を選べる（オフなら API 代はかからない） |
| 見た目 | タスクトレイに常駐。アイコンをクリックすると設定画面が開く |
| ビルド | WSL から Windows 用の exe をクロスコンパイル（`cargo-xwin`） |

## 使い方

`mise run deploy` で `%LOCALAPPDATA%\azarasi-voice` に exe とモデルが置かれる。`azarasi-voice.exe` を起動するとタスクトレイに常駐する。

- Win + 左 Ctrl を押している間だけ録音（トレイのアイコンが赤くなる）。離すと文字にして貼り付ける
- トレイのアイコンは待機中が紺、録音中が赤。「∧」の中に隠れていると見えないので、時計の横までドラッグして常に表示しておく
- トレイのアイコンを左クリックで設定画面、右クリックでメニュー（設定を開く / 終了）
- 設定は `%APPDATA%\azarasi-voice\config.toml`。設定画面で保存すると、次にホットキーを押したときに反映される
- 設定画面の「Windows の起動時に自動で起動する」で、`HKCU\...\CurrentVersion\Run` に登録・解除する
- ログを見たいときは `azarasi-voice.exe --console` で起動する

設定画面（`azarasi-voice.exe --settings`）は別プロセスで、閉じるとプロセスごと終了する。Slint のソフトウェアレンダラーで描画するので GPU は使わない。

### 計測結果（Windows、Ryzen 9 9900X3D、4 スレッド）

| 項目 | 結果 |
|---|---|
| モデル読み込み | 約 1.2 秒 |
| 13 秒の音声の認識 | 約 0.3 秒 |
| メモリ（モデル読み込み中） | 約 700〜800 MB |
| メモリ（モデル解放後） | 約 30〜40 MB |

`cargo run --release --bin bench -- <モデルのディレクトリ> <スレッド数> <wav>...` で再計測できる。

### 注意：スマート アプリ コントロール

Windows のスマート アプリ コントロールがオンだと、署名のない自作 exe は「Device Guard ポリシーによってブロック」されて起動できない。個人では署名の手段が現実的でないため、オフにして使う。

## 開発環境のセットアップ（WSL）

Rust と cargo-xwin のバージョンは `mise.toml` で管理している。ただし clang / lld は mise では入れられないので、apt で入れる。

```sh
# 1. clang / lld（cargo-xwin が Windows 向けのリンクと C/C++ のビルドに使う）
sudo apt install clang lld llvm

# 2. Rust・Windows ターゲット・cargo-xwin
mise install

# 3. ビルド（初回は MSVC CRT と Windows SDK のダウンロードで数分かかる）
mise run build
```

Anthropic API キーは `.env` に `ANTHROPIC_API_KEY=...` と書く（`.gitignore` 済み）。

### モデルのダウンロード

```sh
mkdir -p models && cd models
curl -LO https://github.com/k2-fsa/sherpa-onnx/releases/download/asr-models/sherpa-onnx-nemo-parakeet-tdt_ctc-0.6b-ja-35000-int8.tar.bz2
tar xjf sherpa-onnx-nemo-parakeet-tdt_ctc-0.6b-ja-35000-int8.tar.bz2   # bzip2 が無ければ sudo apt install bzip2
```
