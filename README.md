# voice-input

Windows 用の音声入力ツール。ホットキーを押している間だけ録音し、ローカルの音声認識で文字にして、Claude で整えてから、開いているウィンドウに貼り付ける。

- 音声認識：ローカル（Haas-ja-v1.0 か Parakeet 日本語版)
- 整形：Claude Haiku（Anthropic API）
- ビルド：WSL から Windows 用の exe をクロスコンパイルする（`cargo-xwin`）

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
