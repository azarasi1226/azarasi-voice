; azarasi-voice のインストーラー（Inno Setup 6）
;
; ISCC /DAppVersion=0.1.0 /DExePath=...\azarasi-voice.exe /DIconPath=...\azarasi-voice.ico azarasi-voice.iss
;
; - 管理者権限なしで %LOCALAPPDATA%\Programs\azarasi-voice にインストールする
; - アンインストール時に、自動起動の登録・設定（%APPDATA%）・モデル（%LOCALAPPDATA%）も消す
; - azarasi-voice が起動中なら、インストール・アンインストールの前に終了を促す

#ifndef AppVersion
  #define AppVersion "0.0.0"
#endif
#ifndef ExePath
  #define ExePath "..\target\x86_64-pc-windows-msvc\release\azarasi-voice.exe"
#endif

[Setup]
; AppId はアップデートとアンインストールの識別に使うので、変えてはいけない
AppId={{630F86BE-F4B7-48DC-9A58-4C908DD444B9}
AppName=azarasi-voice
AppVersion={#AppVersion}
AppPublisher=azarasi1226
AppPublisherURL=https://github.com/azarasi1226/azarasi-voice
AppSupportURL=https://github.com/azarasi1226/azarasi-voice/issues
PrivilegesRequired=lowest
DefaultDirName={autopf}\azarasi-voice
DisableProgramGroupPage=yes
DisableDirPage=yes
LicenseFile=..\LICENSE
; 本体の二重起動防止と同じ名前。起動中ならインストール・アンインストール前に終了を促す
AppMutex=Local\azarasi-voice,Local\azarasi-voice-settings
UninstallDisplayIcon={app}\azarasi-voice.exe
UninstallDisplayName=azarasi-voice
OutputDir=Output
OutputBaseFilename=azarasi-voice-setup-{#AppVersion}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
#ifdef IconPath
SetupIconFile={#IconPath}
#endif

[Languages]
Name: "japanese"; MessagesFile: "compiler:Languages\Japanese.isl"

[Tasks]
Name: "autostart"; Description: "Windows の起動時に自動で起動する"

[Files]
Source: "{#ExePath}"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{autoprograms}\azarasi-voice"; Filename: "{app}\azarasi-voice.exe"

[Registry]
; インストール時に「自動で起動する」を選んだら登録する
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: string; ValueName: "azarasi-voice"; ValueData: """{app}\azarasi-voice.exe"""; Tasks: autostart
; 後から設定画面で登録された場合も含め、アンインストール時には必ず消す
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueType: none; ValueName: "azarasi-voice"; Flags: uninsdeletevalue

[Run]
Filename: "{app}\azarasi-voice.exe"; Description: "azarasi-voice を起動する"; Flags: nowait postinstall

[UninstallDelete]
; 設定（%APPDATA%\azarasi-voice）とモデル（%LOCALAPPDATA%\azarasi-voice）
Type: filesandordirs; Name: "{userappdata}\azarasi-voice"
Type: filesandordirs; Name: "{localappdata}\azarasi-voice"
