# azarasi-voice

**Rust 製の、軽量・高速なローカル音声入力ツール（Windows）**

Win + 左 Ctrl を押している間だけ話せば、キーを離した瞬間に文字になって、開いているウィンドウに貼り付けられる。GPU を一切使わないので、ゲームをしながらでも気にせず使える。

- ⚡ **高速**：13 秒の音声を約 0.3 秒で文字にする。CPU だけで動く
- 🪶 **軽量**：待機中のメモリは 30〜40 MB、VRAM は 0 MB
- 🇯🇵 **日本語専用モデル**：NVIDIA が日本語だけで学習させた [Parakeet 日本語版](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja) を使う
- 🔒 **完全ローカル**：音声も文字も外部に送らない。無料・オープンソース

[📚 ドキュメントサイト](https://azarasi1226.github.io/azarasi-voice/)（仕組みと開発）

## インストール

1. [Releases](https://github.com/azarasi1226/azarasi-voice/releases/latest) から `azarasi-voice-setup-<バージョン>.exe` をダウンロードして実行する
   - 「Windows によって PC が保護されました」と出たら「詳細情報」→「実行」
   - 管理者権限は要らない。`%LOCALAPPDATA%\Programs\azarasi-voice` にインストールされ、スタートメニューに登録される
   - 「Windows の起動時に自動で起動する」を選べる（後から設定画面でも変えられる）
2. インストールが終わると起動する。初回だけ、音声認識モデル（約 655 MB）のダウンロード画面が出る。終わると自動でタスクトレイに常駐する
3. タスクトレイの「∧」の中にあるあざらしのアイコンを、時計の横までドラッグしておくと、録音中かどうかがひと目でわかる。アイコンをクリックすると設定画面が開く

アンインストールは Windows の「設定」→「アプリ」から。設定と音声認識モデルも一緒に消える。

**注意**

- Windows のスマート アプリ コントロールがオンだと、署名のない exe は起動できない。[SignPath Foundation](https://signpath.org/) のオープンソース向けの無料署名を申請する予定
- 管理者権限で動いているウィンドウ（一部のゲームなど）には貼り付けられない（Windows の仕組み上の制限）

## プライバシー

azarasi-voice は、初回起動時の音声認識モデルのダウンロード（このリポジトリの GitHub Releases から）を除いて、ネットワークに接続しない。録音した音声も、認識した文字も、外部には送らない。

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it. The only network access is the one-time download of the speech recognition model from this repository's GitHub Releases.

## Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/)（申請予定）

- Committers and reviewers: [azarasi1226](https://github.com/azarasi1226)
- Approvers: [azarasi1226](https://github.com/azarasi1226)

署名する exe は、このリポジトリのソースから GitHub Actions で自動ビルドしたものだけ。

## 計測例（Ryzen 9 9900X3D、4 スレッド）

| 項目 | 結果 |
|---|---|
| モデル読み込み | 約 1.2 秒（ホットキーを押した瞬間に始まり、話している間に終わる） |
| 13 秒の音声の認識 | 約 0.3 秒 |
| メモリ（モデル読み込み中） | 約 700〜800 MB |
| メモリ（モデル解放後） | 約 30〜40 MB |

## 仕組みと開発

[ドキュメントサイト](https://azarasi1226.github.io/azarasi-voice/)にまとめている（リポジトリの [`docs/`](docs/) にある HTML）。

- [全体の構成](https://azarasi1226.github.io/azarasi-voice/overview.html)、[録音から貼り付けまで](https://azarasi1226.github.io/azarasi-voice/recording.html)、[音声認識モデル](https://azarasi1226.github.io/azarasi-voice/model.html)、[設定と画面](https://azarasi1226.github.io/azarasi-voice/settings.html)
- [開発環境の構築](https://azarasi1226.github.io/azarasi-voice/development.html)、[リリースと配布](https://azarasi1226.github.io/azarasi-voice/release.html)

## ライセンス

[GPL-3.0](LICENSE)

使っている主なもの：

- [Slint](https://slint.dev/)（GPL-3.0 で利用）
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)（Apache-2.0）
- 音声認識モデル [nvidia/parakeet-tdt_ctc-0.6b-ja](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)（CC-BY-4.0、NVIDIA）。sherpa-onnx による int8 ONNX 変換版を [Releases](https://github.com/azarasi1226/azarasi-voice/releases/tag/model-parakeet-ja-int8-v1) で再配布している
