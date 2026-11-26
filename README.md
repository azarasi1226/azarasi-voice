# azarasi-voice

Windows 用の音声入力ツール。ホットキーを押している間だけ録音し、ローカルの音声認識で文字にして、開いているウィンドウに貼り付ける。

- **Win + 左 Ctrl を押している間だけ録音**し、離すと文字にして貼り付ける
- 音声認識はすべて PC の中で行う（NVIDIA の [Parakeet 日本語版](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)）。音声を外部に送らない
- **GPU / VRAM を使わない**。ゲームと並行して使っても邪魔をしない
- 待機中は数十 MB。話したときだけモデルを読み込み、しばらく使わなければメモリから外す
- タスクトレイに常駐し、録音中はアイコンが赤くなる

## インストール

1. [Releases](https://github.com/azarasi1226/azarasi-voice/releases/latest) から `azarasi-voice.exe` をダウンロードし、好きな場所に置く
2. `azarasi-voice.exe` を起動する
   - 「Windows によって PC が保護されました」と出たら「詳細情報」→「実行」
3. 初回だけ、音声認識モデル（約 655 MB）のダウンロード画面が出る。終わると自動でタスクトレイに常駐する
4. タスクトレイの「∧」の中にあるあざらしのアイコンを、時計の横までドラッグしておくと、録音中かどうかがひと目でわかる

**スマート アプリ コントロールについて**：Windows のスマート アプリ コントロールがオンだと、署名のない exe は起動できない。[SignPath Foundation](https://signpath.org/) のオープンソース向けの無料署名を申請する予定。

## 使い方

- **Win + 左 Ctrl** を押している間だけ録音する（アイコンが赤くなる）。離すと文字にして、開いているウィンドウに貼り付ける
- 貼り付けはクリップボード経由。直前にコピーしていた文字や画像は、貼り付けたあとに元に戻す
- トレイのアイコンを左クリックで設定画面、右クリックでメニュー（設定を開く / 終了）
- 設定画面でできること
  - 音声認識に使う CPU スレッド数
  - モデルをメモリから外すまでの時間
  - Windows の起動時に自動で起動する
- 管理者権限で動いているウィンドウには貼り付けられない（Windows の仕組み上の制限）

## アンインストール

1. 設定画面で「Windows の起動時に自動で起動する」のチェックを外して保存する
2. トレイのアイコンを右クリック →「終了」
3. 次のフォルダを削除する
   - `azarasi-voice.exe` を置いた場所
   - `%LOCALAPPDATA%\azarasi-voice`（音声認識モデル）
   - `%APPDATA%\azarasi-voice`（設定）

## プライバシー

azarasi-voice は、初回起動時の音声認識モデルのダウンロード（このリポジトリの GitHub Releases から）を除いて、ネットワークに接続しない。録音した音声も、認識した文字も、外部には送らない。

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it. The only network access is the one-time download of the speech recognition model from this repository's GitHub Releases.

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/)（申請予定）

- Committers and reviewers: [azarasi1226](https://github.com/azarasi1226)
- Approvers: [azarasi1226](https://github.com/azarasi1226)

署名する exe は、このリポジトリのソースから GitHub Actions で自動ビルドしたものだけ。

## 仕組み

| 項目 | 内容 |
|---|---|
| 音声認識 | Parakeet 日本語版（`parakeet-tdt_ctc-0.6b-ja` の int8 ONNX 変換版）を sherpa-onnx で CPU 実行 |
| モデル | `%LOCALAPPDATA%\azarasi-voice\model`。初回起動時に [Releases](https://github.com/azarasi1226/azarasi-voice/releases/tag/model-parakeet-ja-int8-v1) からダウンロードし、SHA-256 で中身を確認する |
| 設定 | `%APPDATA%\azarasi-voice\config.toml`。設定画面で保存すると、次にホットキーを押したときに反映される |
| 自動起動 | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` に登録・解除する |
| 画面 | 設定画面とダウンロード画面は別プロセスで、閉じるとプロセスごと終了する。Slint のソフトウェアレンダラーで描画するので GPU は使わない |

### 計測結果（Ryzen 9 9900X3D、4 スレッド）

| 項目 | 結果 |
|---|---|
| モデル読み込み | 約 1.2 秒（ホットキーを押した瞬間に始まり、話している間に終わる） |
| 13 秒の音声の認識 | 約 0.3 秒 |
| メモリ（モデル読み込み中） | 約 700〜800 MB |
| メモリ（モデル解放後） | 約 30〜40 MB |

## 開発

WSL から Windows 用の exe をクロスコンパイルする（`cargo-xwin`）。Rust と cargo-xwin のバージョンは `mise.toml` で管理している。clang / lld は mise では入れられないので apt で入れる。

```sh
# 1. clang / lld / llvm-rc（cargo-xwin が Windows 向けのリンクと C/C++ のビルドに、build.rs がアイコンの埋め込みに使う）
sudo apt install clang lld llvm

# 2. Rust・Windows ターゲット・cargo-xwin
mise install

# 3. ビルド（初回は MSVC CRT と Windows SDK のダウンロードで数分かかる）
mise run build

# 4. Windows のダウンロードフォルダの azarasi-voice-dev に exe を配置（起動中でも置き換えられる）
mise run deploy
```

- `mise run fmt` で整形、`mise run lint` で整形のずれと clippy の警告をチェックする。CI でも同じチェックを行い、警告が 1 つでもあると失敗する

- ログを見たいときは `azarasi-voice.exe --console` で起動する
- 速度とメモリは `cargo run --release --bin bench -- <モデルのディレクトリ> <スレッド数> <wav>...` で計測できる
- 初回セットアップ画面の動作確認は、環境変数 `AZARASI_VOICE_MODEL_DIR` に空のフォルダを指定して `azarasi-voice.exe --setup` を起動する

## ライセンス

[GPL-3.0](LICENSE)

使っている主なもの：

- [Slint](https://slint.dev/)（GPL-3.0 で利用）
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)（Apache-2.0）
- 音声認識モデル [nvidia/parakeet-tdt_ctc-0.6b-ja](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)（CC-BY-4.0、NVIDIA）。sherpa-onnx による int8 ONNX 変換版を [Releases](https://github.com/azarasi1226/azarasi-voice/releases/tag/model-parakeet-ja-int8-v1) で再配布している
