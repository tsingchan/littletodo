# Windows desktop feasibility probe

Isolated C# / .NET Framework probe, not LittleTodo and not a Tauri implementation. Uses installed Windows compiler/runtime, no downloaded dependency. No app data, services, autostart, system hooks or notifications.

Compile on Windows from repository root:

```powershell
$probeSource = Join-Path (Get-Location) 'probes\windows-desktop\DesktopProbe.cs'
$probeOutput = Join-Path (Get-Location) 'probes\windows-desktop\DesktopProbe.exe'
& "$env:WINDIR\Microsoft.NET\Framework64\v4.0.30319\csc.exe" /nologo /target:winexe "/out:$probeOutput" /reference:System.Windows.Forms.dll /reference:System.Drawing.dll $probeSource
```

Run DesktopProbe.exe. Attach only reparents its own panel into the existing desktop icon host, without sending undocumented shell messages or changing desktop icons. Controller remains a normal window. Exit probe disposes panel. Restart Explorer during the probe is not tested.

Test attach, checkbox, header drag, detail text edit, opacity/background, ordinary-app occlusion and show-desktop retention. A parent handle/visible flag alone does not prove on-screen visibility or clickability. Log records actual UI events and settings, not visual success. Generated exe/log are local artifacts, not release assets.
