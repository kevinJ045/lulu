# Getting Started

This guide will walk you through installing Lulu on your system.

## Linux & macOS

For most Linux distributions(probs macos), you can install Lulu by running this:

```bash
curl -fsSL https://raw.githubusercontent.com/kevinj045/lulu/main/install-linux.sh | bash
```

This script will download the latest release, unpack it, and install the `lulu` executable into your user's binary path (`~/.local/bin`). You may need to add this directory to your shell's `PATH` variable if it isn't already.

### Manual Installation

Alternatively, you can download the appropriate archive for your system from the [Latest Release page](https://github.com/kevinJ045/lulu/releases/latest), extract the `lulu` executable, and place it in a directory included in your `PATH` (e.g., `/usr/local/bin`).

## Windows

On Windows, you can install Lulu by opening PowerShell and running the following command:

```powershell
irm https://raw.githubusercontent.com/kevinj045/lulu/main/install-windows.ps1 | iex
```

This will install the `lulu.exe` executable and add it to your user's `PATH`.

### Manual Installation

You can also download the `.exe` installer or `.exe` itself from the [Latest Release page](https://github.com/kevinJ045/lulu/releases/latest) and run it.

## Verifying Installation

After installation, you can verify that Lulu is working by running:

```bash
lulu --version
```

This should print the installed version of Lulu. You're now ready to start your first project!
