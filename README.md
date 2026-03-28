# GliderRust

Windows-only single-process Rust rebuild of MMOGlider.

## Current status

- Project skeleton and AI-first module boundaries created.
- Initial profile XML parser and tests included.
- Memory layer currently read-only stub; Win32 attach/read implementation is next.

## Quick start

1. Install Rust with rustup.
2. Install Visual Studio 2022 Build Tools with the C++ workload.
3. Open a terminal and import the VS build environment:

```powershell
$vsdev = 'C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\Common7\Tools\VsDevCmd.bat'
cmd.exe /c "`\"$vsdev`\" -no_logo && set" | ForEach-Object {
	if ($_ -match '^(.*?)=(.*)$') {
		Set-Item -Path "Env:$($matches[1])" -Value $matches[2]
	}
}
```

4. Run `cargo test`.
5. Run `cargo run`.
