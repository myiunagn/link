import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { LanguageClient, LanguageClientOptions, ServerOptions, TransportKind, Trace } from 'vscode-languageclient/node';

let client: LanguageClient | undefined;

export function activate(context: vscode.ExtensionContext) {
    const binaryPath = resolveLinkBinary();
    if (!binaryPath) {
        vscode.window.showWarningMessage(
            'Link language server not found. Build with `cargo build` or set "link.binaryPath".'
        );
        return;
    }

    const serverOptions: ServerOptions = {
        command: binaryPath,
        args: ['lsp'],
        transport: TransportKind.stdio,
    };

    const traceChannel = vscode.window.createOutputChannel('Link Language Server Trace');
    context.subscriptions.push(traceChannel);

    const clientOptions: LanguageClientOptions = {
        documentSelector: [{ scheme: 'file', language: 'link' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.link'),
        },
        outputChannel: vscode.window.createOutputChannel('Link Language Server'),
        traceOutputChannel: traceChannel,
    };

    client = new LanguageClient(
        'linkLanguageServer',
        'Link Language Server',
        serverOptions,
        clientOptions,
    );

    // Apply initial trace setting.
    applyTraceSetting(client);

    context.subscriptions.push(
        vscode.workspace.onDidChangeConfiguration(e => {
            if (e.affectsConfiguration('link.trace.server') && client) {
                applyTraceSetting(client);
            }
        })
    );

    // LanguageClient implements Disposable; register it so VSCode stops the
    // server automatically on extension deactivation.
    context.subscriptions.push(client);
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

function applyTraceSetting(c: LanguageClient) {
    const value = vscode.workspace.getConfiguration('link').get<string>('trace.server') || 'off';
    const trace = Trace.fromString(value);
    c.setTrace(trace);
}

/// Locate the `link` binary. Resolution order:
///   1. `link.binaryPath` setting (if set and exists)
///   2. `<workspace>/target/debug/link[.exe]`
///   3. `<workspace>/target/release/link[.exe]`
///   4. `link` on PATH
function resolveLinkBinary(): string | undefined {
    const isWin = process.platform === 'win32';
    const exe = isWin ? 'link.exe' : 'link';

    const configPath = vscode.workspace.getConfiguration('link').get<string>('binaryPath');
    if (configPath && fs.existsSync(configPath)) {
        return configPath;
    }

    const folders = vscode.workspace.workspaceFolders;
    if (folders) {
        for (const folder of folders) {
            const debug = path.join(folder.uri.fsPath, 'target', 'debug', exe);
            if (fs.existsSync(debug)) { return debug; }
            const release = path.join(folder.uri.fsPath, 'target', 'release', exe);
            if (fs.existsSync(release)) { return release; }
        }
    }

    return exe;
}
