"use strict";
var __createBinding = (this && this.__createBinding) || (Object.create ? (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    var desc = Object.getOwnPropertyDescriptor(m, k);
    if (!desc || ("get" in desc ? !m.__esModule : desc.writable || desc.configurable)) {
      desc = { enumerable: true, get: function() { return m[k]; } };
    }
    Object.defineProperty(o, k2, desc);
}) : (function(o, m, k, k2) {
    if (k2 === undefined) k2 = k;
    o[k2] = m[k];
}));
var __setModuleDefault = (this && this.__setModuleDefault) || (Object.create ? (function(o, v) {
    Object.defineProperty(o, "default", { enumerable: true, value: v });
}) : function(o, v) {
    o["default"] = v;
});
var __importStar = (this && this.__importStar) || (function () {
    var ownKeys = function(o) {
        ownKeys = Object.getOwnPropertyNames || function (o) {
            var ar = [];
            for (var k in o) if (Object.prototype.hasOwnProperty.call(o, k)) ar[ar.length] = k;
            return ar;
        };
        return ownKeys(o);
    };
    return function (mod) {
        if (mod && mod.__esModule) return mod;
        var result = {};
        if (mod != null) for (var k = ownKeys(mod), i = 0; i < k.length; i++) if (k[i] !== "default") __createBinding(result, mod, k[i]);
        __setModuleDefault(result, mod);
        return result;
    };
})();
Object.defineProperty(exports, "__esModule", { value: true });
exports.activate = activate;
exports.deactivate = deactivate;
const vscode = __importStar(require("vscode"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
const node_1 = require("vscode-languageclient/node");
let client;
function activate(context) {
    const binaryPath = resolveLinkBinary();
    if (!binaryPath) {
        vscode.window.showWarningMessage('Link language server not found. Build with `cargo build` or set "link.binaryPath".');
        return;
    }
    const serverOptions = {
        command: binaryPath,
        args: ['lsp'],
        transport: node_1.TransportKind.stdio,
    };
    const traceChannel = vscode.window.createOutputChannel('Link Language Server Trace');
    context.subscriptions.push(traceChannel);
    const clientOptions = {
        documentSelector: [{ scheme: 'file', language: 'link' }],
        synchronize: {
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.link'),
        },
        outputChannel: vscode.window.createOutputChannel('Link Language Server'),
        traceOutputChannel: traceChannel,
    };
    client = new node_1.LanguageClient('linkLanguageServer', 'Link Language Server', serverOptions, clientOptions);
    // Apply initial trace setting.
    applyTraceSetting(client);
    context.subscriptions.push(vscode.workspace.onDidChangeConfiguration(e => {
        if (e.affectsConfiguration('link.trace.server') && client) {
            applyTraceSetting(client);
        }
    }));
    // LanguageClient implements Disposable; register it so VSCode stops the
    // server automatically on extension deactivation.
    context.subscriptions.push(client);
    client.start();
}
function deactivate() {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
function applyTraceSetting(c) {
    const value = vscode.workspace.getConfiguration('link').get('trace.server') || 'off';
    const trace = node_1.Trace.fromString(value);
    c.setTrace(trace);
}
/// Locate the `link` binary. Resolution order:
///   1. `link.binaryPath` setting (if set and exists)
///   2. `<workspace>/target/debug/link[.exe]`
///   3. `<workspace>/target/release/link[.exe]`
///   4. `link` on PATH
function resolveLinkBinary() {
    const isWin = process.platform === 'win32';
    const exe = isWin ? 'link.exe' : 'link';
    const configPath = vscode.workspace.getConfiguration('link').get('binaryPath');
    if (configPath && fs.existsSync(configPath)) {
        return configPath;
    }
    const folders = vscode.workspace.workspaceFolders;
    if (folders) {
        for (const folder of folders) {
            const debug = path.join(folder.uri.fsPath, 'target', 'debug', exe);
            if (fs.existsSync(debug)) {
                return debug;
            }
            const release = path.join(folder.uri.fsPath, 'target', 'release', exe);
            if (fs.existsSync(release)) {
                return release;
            }
        }
    }
    return exe;
}
//# sourceMappingURL=extension.js.map