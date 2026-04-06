# Deslop Finding Opener

This extension parses finding reports that contain `Source: /path/to/file:line` entries and opens each location in a VS Code tab.

When a text file contains a matching `Source:` line, the extension shows a clickable inline code lens with a link icon. Clicking it opens the target file in a tab at the reported line.
Each `Source:` occurrence is treated independently, so repeated findings for the same path still get their own clickable target.

## Usage

1. Copy a finding block to your clipboard.
2. Run `Deslop: Open Findings From Clipboard`.
3. The extension opens each detected source location in a VS Code tab at the reported line.

## Supported input

The parser looks for lines like:

```text
Source: /home/chinmay/ChinmayPersonalProjects/gopdfsuit/internal/handlers/handlers.go:164
```
