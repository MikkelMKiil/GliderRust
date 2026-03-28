# GliderRust

Windows-only single-process Rust rebuild of MMOGlider.

## Current status

- Project skeleton and AI-first module boundaries created.
- Initial profile XML parser and tests included.
- Memory layer includes read-only Win32 attach/read telemetry for WotLK 3.3.5a.

## Memory reading rule

- Memory reads must be deterministic single-path reads.
- Do not add fallback offsets, probe lists, default-value substitutions, or alternate pointer-chain fallbacks in memory code.
- When a read fails, surface the failure in diagnostics/errors and fix the canonical offset/path instead of trying alternatives.

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
