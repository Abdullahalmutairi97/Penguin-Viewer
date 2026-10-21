; Windows installer (Inno Setup). Built by .github/workflows/release.yml:
;   iscc /DVersion=0.1.0 /DDist=C:\path\to\dist packaging\windows\penguin-viewer.iss
; Installs for the current user only (no admin rights) and offers itself under "Open with"
; for every picture and video type it reads. The extension list comes from src/files.rs.

[Setup]
AppId={{9B3E7C21-5A4F-4D8E-A1C6-2F7B9E0D4C58}
AppName=Penguin Viewer
AppVersion={#Version}
AppPublisher=Penguin Viewer
AppPublisherURL=https://github.com/Abdullahalmutairi97/Penguin-Viewer
DefaultDirName={autopf}\Penguin Viewer
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes
OutputDir={#Dist}
OutputBaseFilename=PenguinViewer-Setup
SetupIconFile=..\..\assets\icon.ico
UninstallDisplayIcon={app}\penguin-viewer.exe
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: "desktopicon"; Description: "Put Penguin Viewer on the desktop"

[Files]
Source: "{#Dist}\penguin-viewer\*"; DestDir: "{app}"; Flags: recursesubdirs ignoreversion

[Icons]
Name: "{autoprograms}\Penguin Viewer"; Filename: "{app}\penguin-viewer.exe"
Name: "{autodesktop}\Penguin Viewer"; Filename: "{app}\penguin-viewer.exe"; Tasks: desktopicon

[Registry]
Root: HKA; Subkey: "Software\Classes\PenguinViewer.File"; ValueType: string; ValueName: ""; ValueData: "Penguin Viewer file"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\PenguinViewer.File\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\penguin-viewer.exe,0"
Root: HKA; Subkey: "Software\Classes\PenguinViewer.File\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\penguin-viewer.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe"; ValueType: string; ValueName: "FriendlyAppName"; ValueData: "Penguin Viewer"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\penguin-viewer.exe"" ""%1"""
Root: HKA; Subkey: "Software\Classes\.png\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".png"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.jpg\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".jpg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.jpeg\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".jpeg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.jfif\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".jfif"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.webp\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".webp"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.gif\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".gif"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.bmp\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".bmp"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.ico\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".ico"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.tif\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".tif"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.tiff\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".tiff"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.tga\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".tga"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.qoi\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".qoi"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.pnm\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".pnm"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.pbm\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".pbm"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.pgm\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".pgm"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.ppm\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".ppm"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.hdr\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".hdr"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.exr\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".exr"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.dds\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".dds"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.ff\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".ff"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.avif\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".avif"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.svg\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".svg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.svgz\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".svgz"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mp4\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mp4"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.m4v\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".m4v"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mkv\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mkv"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.webm\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".webm"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mov\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mov"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.avi\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".avi"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.wmv\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".wmv"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.flv\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".flv"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.f4v\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".f4v"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.ogv\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".ogv"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mpg\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mpg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mpeg\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mpeg"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.m2ts\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".m2ts"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.mts\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".mts"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.3gp\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".3gp"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.divx\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".divx"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.asf\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".asf"; ValueData: ""
Root: HKA; Subkey: "Software\Classes\.vob\OpenWithProgids"; ValueType: string; ValueName: "PenguinViewer.File"; ValueData: ""; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\Applications\penguin-viewer.exe\SupportedTypes"; ValueType: string; ValueName: ".vob"; ValueData: ""

[Run]
Filename: "{app}\penguin-viewer.exe"; Description: "Open Penguin Viewer"; Flags: nowait postinstall skipifsilent
