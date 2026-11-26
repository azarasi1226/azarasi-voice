# azarasi-voice

**Rust 製の Windows 専用音声入力ツール**

- ⚡ **軽量・高速**：待機中のメモリは 30〜40 MB
- 🎮 **ゲームと一緒に使える**：GPU を一切使わない（VRAM 0 MB）
- 🇯🇵 **高精度な日本語入力**：NVIDIA の [Parakeet 日本語版](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)
- 🔒 **完全ローカル**：音声を外部に送らない。無料・オープンソース

[📚 ドキュメントサイト](https://azarasi1226.github.io/azarasi-voice/)

## インストール

1. [Releases](https://github.com/azarasi1226/azarasi-voice/releases/latest) から `azarasi-voice-setup-<バージョン>.exe` をダウンロードして実行する
   - 「Windows によって PC が保護されました」と出たら「詳細情報」→「実行」
2. インストールが終わると起動する。初回だけ、音声認識モデル（約 655 MB）のダウンロード画面が出る。終わると自動でタスクトレイに常駐する
3. タスクトレイの「∧」の中にあるあざらしのアイコンを、時計の横までドラッグしておくと、録音中かどうかがひと目でわかる。アイコンをクリックすると設定画面が開く
4. <kbd>Win</kbd> + <kbd>左 Ctrl</kbd> を押しながら話し、キーを離すと、文字になって開いているウィンドウに貼り付けられる

**注意**

- Windows のスマート アプリ コントロールがオンだと、署名のない exe は起動できない。[SignPath Foundation](https://signpath.org/) のオープンソース向けの無料署名を申請する予定
- 管理者権限で動いているウィンドウ（一部のゲームなど）には貼り付けられない（Windows の仕組み上の制限）

## 計測例（Ryzen 9 9900X3D、4 スレッド）

| 項目 | 結果 |
|---|---|
| モデル読み込み | 約 1.2 秒（ホットキーを押した瞬間に始まり、話している間に終わる） |
| 13 秒の音声の認識 | 約 0.3 秒 |
| メモリ（モデル読み込み中） | 約 700〜800 MB |
| メモリ（モデル解放後） | 約 30〜40 MB |

## ライセンス

[GPL-3.0](LICENSE)。コード署名とプライバシーについては [Code signing policy](CODE_SIGNING_POLICY.md) を参照。

使っている主なもの：

- [Slint](https://slint.dev/)（GPL-3.0 で利用）
- [sherpa-onnx](https://github.com/k2-fsa/sherpa-onnx)（Apache-2.0）
- 音声認識モデル [nvidia/parakeet-tdt_ctc-0.6b-ja](https://huggingface.co/nvidia/parakeet-tdt_ctc-0.6b-ja)（CC-BY-4.0、NVIDIA）。sherpa-onnx による int8 ONNX 変換版を [Releases](https://github.com/azarasi1226/azarasi-voice/releases/tag/model-parakeet-ja-int8-v1) で再配布している
