# Code signing policy

Free code signing provided by [SignPath.io](https://about.signpath.io/), certificate by [SignPath Foundation](https://signpath.org/)（申請予定）

- Committers and reviewers: [azarasi1226](https://github.com/azarasi1226)
- Approvers: [azarasi1226](https://github.com/azarasi1226)

署名する exe は、このリポジトリのソースから GitHub Actions で自動ビルドしたものだけ。

## Privacy / プライバシー

azarasi-voice は、初回起動時の音声認識モデルのダウンロード（このリポジトリの GitHub Releases から）を除いて、ネットワークに接続しない。録音した音声も、認識した文字も、外部には送らない。

This program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it. The only network access is the one-time download of the speech recognition model from this repository's GitHub Releases.
