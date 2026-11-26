# azarasi-voice

Windows 用の音声入力ツール。ホットキーを押している間だけ録音し、ローカルの音声認識で文字にして、開いているウィンドウに貼り付ける。

[📚 ドキュメントサイト](https://azarasi1226.github.io/azarasi-voice/)（仕組みと開発）

- **Win + 左 Ctrl を押している間だけ録音**し、離すと文字にして貼り付ける
- 音声認識はすべて PC の中で行う（NVIDIA の [Parakeet 日本語版](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)）。音声を外部に送らない
- **GPU / VRAM を使わない**。ゲームと並行して使っても邪魔をしない
- 待機中は数十 MB。話したときだけモデルを読み込み、しばらく使わなければメモリから外す
- タスクトレイに常駐し、録音中はアイコンが赤くなる

## なぜ作ったか

音声入力ソフトを探すと、だいたい次のどちらかになる。

- **精度の高いものは有料が多い**。クラウドで処理するものは、音声を外部のサーバーに送ることにもなる
- **無料でローカルで動くものは、GPU のメモリ（VRAM）を大量に使うものが多い**。Whisper large のような大きなモデルを GPU に載せるため、数 GB の VRAM を使い続ける。ゲームをしながら使うと、ゲームと VRAM を取り合ってしまう

azarasi-voice は、無料で、ローカルで、しかもゲームの邪魔をしない音声入力を目指して作った。

- **日本語専用のモデル**：NVIDIA が日本語だけで学習させた音声認識モデル [Parakeet 日本語版](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja) を使う。多くの言語を 1 つでこなすモデルと違い、日本語に特化している
- **CPU だけで速い**：モデルを軽量化（int8 量子化）して CPU で動かす。13 秒の音声を約 0.3 秒で文字にする（下の計測例を参照）。話し終わってから貼り付けまで、ほとんど待たない
- **GPU を使わない**：VRAM は 0 MB。GPU はゲームに全部使える
- **普段は軽い**：待機中のメモリは 30〜40 MB ほど。モデル（約 700 MB）はホットキーを押した瞬間に読み込み始め（話している間に終わる）、しばらく使わなければメモリから解放する
- **無料・オープンソース**：音声も文字も外部に送らない

## インストール

1. [Releases](https://github.com/azarasi1226/azarasi-voice/releases/latest) から `azarasi-voice-setup-<バージョン>.exe` をダウンロードして実行する
   - 「Windows によって PC が保護されました」と出たら「詳細情報」→「実行」
   - 管理者権限は要らない。`%LOCALAPPDATA%\Programs\azarasi-voice` にインストールされ、スタートメニューに登録される
   - 「Windows の起動時に自動で起動する」を選べる（後から設定画面でも変えられる）
2. インストールが終わると起動する。初回だけ、音声認識モデル（約 655 MB）のダウンロード画面が出る。終わると自動でタスクトレイに常駐する
3. タスクトレイの「∧」の中にあるあざらしのアイコンを、時計の横までドラッグしておくと、録音中かどうかがひと目でわかる

**スマート アプリ コントロールについて**：Windows のスマート アプリ コントロールがオンだと、署名のない exe は起動できない。[SignPath Foundation](https://signpath.org/) のオープンソース向けの無料署名を申請する予定。

## 使い方

- **Win + 左 Ctrl** を押している間だけ録音する（アイコンが赤くなる）。離すと文字にして、開いているウィンドウに貼り付ける
- 貼り付けはクリップボード経由。直前にコピーしていた文字や画像は、貼り付けたあとに元に戻す
- トレイのアイコンを左クリックで設定画面、右クリックでメニュー（設定を開く / 終了）
- 録音の開始と終了で短い効果音が鳴る（設定画面でオフにできる）
  - ワイヤレスヘッドセットなどでは、しばらく音を鳴らしていなかったときだけ、開始の音が 0.4 秒ほど遅れて聞こえる（機器を省電力の状態から起こすため）
- 設定画面でできること
  - 音声認識モデルをメモリから解放するまでの時間
  - Windows の起動時に自動で起動する
  - 録音の開始・終了の効果音のオン・オフ
- 音声認識に使う CPU スレッド数は自動で決まる（論理コア数の 1/4、1〜4）。ゲームなどの CPU を奪いにくいよう控えめにしている
- 管理者権限で動いているウィンドウには貼り付けられない（Windows の仕組み上の制限）

## アンインストール

Windows の「設定」→「アプリ」→「インストールされているアプリ」から azarasi-voice をアンインストールする。起動中なら終了を促される。次のものもまとめて消える。

- 自動起動の登録
- 設定（`%APPDATA%\azarasi-voice`）
- 音声認識モデル（`%LOCALAPPDATA%\azarasi-voice`）

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
