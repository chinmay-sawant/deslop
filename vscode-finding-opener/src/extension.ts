import * as vscode from 'vscode';
import { parseFindingLineMatches, parseFindingLocations } from './parser';

const FILENAME_SCHEME = { scheme: 'file' };

async function openLocationInEditor(filePath: string, line: number): Promise<void> {
  const uri = vscode.Uri.file(filePath);
  const document = await vscode.workspace.openTextDocument(uri);
  const editor = await vscode.window.showTextDocument(document, {
    preview: false,
    preserveFocus: false,
    viewColumn: vscode.ViewColumn.Active,
  });

  const targetLine = Math.max(0, line - 1);
  const position = new vscode.Position(targetLine, 0);
  editor.selection = new vscode.Selection(position, position);
  editor.revealRange(new vscode.Range(position, position), vscode.TextEditorRevealType.InCenter);
}

export function activate(context: vscode.ExtensionContext): void {
  const refreshEmitter = new vscode.EventEmitter<void>();

  const openFindingCommand = vscode.commands.registerCommand(
    'deslopFindingOpener.openFindingLocation',
    async (filePath: string, line: number) => {
      try {
        await openLocationInEditor(filePath, line);
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unable to open the target file.';
        vscode.window.showErrorMessage(message);
      }
    },
  );

  const openClipboardCommand = vscode.commands.registerCommand(
    'deslopFindingOpener.openFindings',
    async (explicitText?: string) => {
      try {
        const text = typeof explicitText === 'string' && explicitText.trim().length > 0
          ? explicitText
          : await vscode.env.clipboard.readText();
        const findings = parseFindingLocations(text);

        if (findings.length === 0) {
          vscode.window.showWarningMessage('No Source: path:line entries were found.');
          return;
        }

        for (const finding of findings) {
          await openLocationInEditor(finding.filePath, finding.line);
        }

        vscode.window.showInformationMessage(
          `Opened ${findings.length} finding location${findings.length === 1 ? '' : 's'} in tabs.`,
        );
      } catch (error) {
        const message = error instanceof Error ? error.message : 'Unable to open findings.';
        vscode.window.showErrorMessage(message);
      }
    },
  );

  const codeLensProvider = vscode.languages.registerCodeLensProvider(
    FILENAME_SCHEME,
    {
      onDidChangeCodeLenses: refreshEmitter.event,
      provideCodeLenses(document: vscode.TextDocument): vscode.CodeLens[] {
        const matches = parseFindingLineMatches(document.getText());
        return matches.map(
          (match) =>
            new vscode.CodeLens(
              new vscode.Range(match.lineIndex, 0, match.lineIndex, Math.max(1, match.sourceLine.length)),
              {
                command: 'deslopFindingOpener.openFindingLocation',
                title: '$(link-external) Open in tab',
                arguments: [match.filePath, match.line],
              },
            ),
        );
      },
      resolveCodeLens(codeLens: vscode.CodeLens): vscode.ProviderResult<vscode.CodeLens> {
        return codeLens;
      },
    },
  );

  context.subscriptions.push(
    refreshEmitter,
    vscode.workspace.onDidOpenTextDocument(() => refreshEmitter.fire()),
    vscode.workspace.onDidChangeTextDocument(() => refreshEmitter.fire()),
    openFindingCommand,
    openClipboardCommand,
    codeLensProvider,
  );
}

export function deactivate(): void {
  // Nothing to dispose explicitly.
}
