# ai_project_template

A tiny cross‑platform utility to scaffold a Python project with a friendly GUI and CLI.

- GUI: Rust/Iced (macOS, Windows, Linux)
- Scripts: Python (`create_project.py`) — unchanged
- Features: environment checks (python/pip/venv), flags (--venv/--install/--refresh-templates/--force), execution log

## Download (GUI)
1) Open the repository’s Releases page (GitHub → Releases)
2) Download the binary for your platform:
   - macOS (Apple Silicon): `ai_project_template-Darwin-arm64`
   - macOS (Intel): `ai_project_template-Darwin-x86_64`
   - Windows: `ai_project_template-Windows-x86_64.exe` or `ai_project_template-Windows-arm64.exe`
   - Linux: `ai_project_template-Linux-x86_64` or `ai_project_template-Linux-arm64`
3) Place the binary next to `create_project.py` and run it

Notes:
- macOS: allow the app in System Settings → Privacy & Security
- Windows: SmartScreen may ask for confirmation (“Run anyway”)
- Linux: make the file executable — `chmod +x ./ai_project_template-Linux-*`

## Usage (GUI)
1) Run the app
2) Enter the project name
3) Toggle desired options (--venv, --install, --refresh-templates, --force)
4) Click “Create project”
5) Wait for a short progress (~2s); the log will show “Done” and `ExitCode`

## Usage (CLI)
```bash
python create_project.py "{project_name}" --venv [--install] [--refresh-templates] [--force]
```

## Releases
- GitHub Actions builds releases automatically on tags `v*`
- macOS/Windows/Linux binaries are attached to the Release

## Support
- Ensure `create_project.py` is placed next to the binary
- Verify Python is in PATH (GUI shows hints)
- When asking for help, paste the log and `ExitCode`