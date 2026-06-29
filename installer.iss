#define MyAppName "Bunzo"
#define MyAppPublisher "Ugesh Praavin D"
#define MyAppURL "https://bunzo.dev"
#define MyAppExeName "bzc.exe"

#ifndef MyAppVersion
  #define MyAppVersion "0.8.0-alpha"
#endif

[Setup]
AppId={{B5B821A8-7C32-4DFA-8E2B-5B2E1E1C7E1C}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={commonpf}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
LicenseFile=target\staging-installer\LICENSE
OutputDir=release\installer
OutputBaseFilename=bunzo-{#MyAppVersion}-windows-x64-setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
ArchitecturesAllowed=x64
ArchitecturesInstallIn64BitMode=x64
ChangesEnvironment=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Types]
Name: "full"; Description: "Full installation"
Name: "compact"; Description: "Compact installation"
Name: "custom"; Description: "Custom installation"; Flags: iscustom

[Components]
Name: "main"; Description: "Bunzo Compiler and Toolchain"; Types: full compact custom; Flags: fixed
Name: "docs"; Description: "Documentation"; Types: full
Name: "examples"; Description: "Examples"; Types: full

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "target\staging-installer\bin\*"; DestDir: "{app}\bin"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\toolchain\*"; DestDir: "{app}\toolchain"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\std\*"; DestDir: "{app}\std"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\runtime\*"; DestDir: "{app}\runtime"; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\examples\*"; DestDir: "{app}\examples"; Components: examples; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\docs\*"; DestDir: "{app}\docs"; Components: docs; Flags: ignoreversion recursesubdirs createallsubdirs
Source: "target\staging-installer\LICENSE"; DestDir: "{app}\licenses"; Flags: ignoreversion
Source: "target\staging-installer\README.md"; DestDir: "{app}"; Flags: ignoreversion
Source: "target\staging-installer\CHANGELOG.md"; DestDir: "{app}"; Flags: ignoreversion

[Icons]
Name: "{group}\{#MyAppName} CLI"; Filename: "{sys}\cmd.exe"; Parameters: "/k ""cd /d ""{app}"" && bin\bzc.exe --version"""
Name: "{group}\{#MyAppName} Interactive REPL"; Filename: "{app}\bin\bzc.exe"
Name: "{group}\Uninstall {#MyAppName}"; Filename: "{uninstallexe}"
Name: "{commondesktop}\{#MyAppName} Interactive REPL"; Filename: "{app}\bin\bzc.exe"; Tasks: desktopicon

[Registry]
Root: HKLM; Subkey: "SYSTEM\CurrentControlSet\Control\Session Manager\Environment"; ValueType: expandsz; ValueName: "Path"; ValueData: "{olddata};{app}\bin"; Check: NeedsAddPath; Flags: preservestringtype

[Code]
const
  WM_SETTINGCHANGE = $001A;
  SMTO_ABORTIFHUNG = $0002;

function SendMessageTimeout(
  hWnd: HWND;
  Msg: Cardinal;
  wParam: Cardinal;
  lParam: String;
  fuFlags: Cardinal;
  uTimeout: Cardinal;
  var lpdwResult: DWORD_PTR
): Longint;
external 'SendMessageTimeoutW@user32.dll stdcall';

function NeedsAddPath(): Boolean;
var
  OrigPath: String;
  TargetDir: String;
begin
  TargetDir := ExpandConstant('{app}\bin');
  if not RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath) then
  begin
    Result := True;
    exit;
  end;
  { Check if path already exists }
  Result := Pos(TargetDir, OrigPath) = 0;
end;

procedure RefreshEnvironment();
var
  ResultAddr: DWORD_PTR;
begin
  SendMessageTimeout(HWND_BROADCAST, WM_SETTINGCHANGE, 0, 'Environment', SMTO_ABORTIFHUNG, 5000, ResultAddr);
end;

procedure RemovePath();
var
  OrigPath: String;
  PathToRemove: String;
  PosNum: Integer;
begin
  if RegQueryStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath) then
  begin
    PathToRemove := ';' + ExpandConstant('{app}\bin');
    PosNum := Pos(PathToRemove, OrigPath);
    if PosNum > 0 then
    begin
      Delete(OrigPath, PosNum, Length(PathToRemove));
      RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath);
    end
    else
    begin
      PathToRemove := ExpandConstant('{app}\bin') + ';';
      PosNum := Pos(PathToRemove, OrigPath);
      if PosNum > 0 then
      begin
        Delete(OrigPath, PosNum, Length(PathToRemove));
        RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath);
      end
      else
      begin
        PathToRemove := ExpandConstant('{app}\bin');
        PosNum := Pos(PathToRemove, OrigPath);
        if PosNum > 0 then
        begin
          Delete(OrigPath, PosNum, Length(PathToRemove));
          RegWriteStringValue(HKLM, 'SYSTEM\CurrentControlSet\Control\Session Manager\Environment', 'Path', OrigPath);
        end;
      end;
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  BinInstalled: Boolean;
  ToolchainInstalled: Boolean;
  StdInstalled: Boolean;
  RuntimeInstalled: Boolean;
begin
  if CurStep = ssPostInstall then
  begin
    RefreshEnvironment();
    
    // Environment Validation
    BinInstalled := FileExists(ExpandConstant('{app}\bin\bzc.exe'));
    ToolchainInstalled := FileExists(ExpandConstant('{app}\toolchain\bin\clang.exe'));
    StdInstalled := DirExists(ExpandConstant('{app}\std'));
    RuntimeInstalled := DirExists(ExpandConstant('{app}\runtime'));
    
    if not (BinInstalled and ToolchainInstalled and StdInstalled and RuntimeInstalled) then
    begin
      MsgBox('Installation validation failed. Please verify that all components were installed correctly.', mbError, MB_OK);
    end;
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemovePath();
    RefreshEnvironment();
  end;
end;
