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
const child_process = __importStar(require("child_process"));
const path = __importStar(require("path"));
const fs = __importStar(require("fs"));
let diagnosticCollection;
let debounceTimer = null;
function activate(context) {
    diagnosticCollection = vscode.languages.createDiagnosticCollection('link');
    vscode.workspace.onDidChangeTextDocument((e) => {
        if (e.document.languageId === 'link') {
            scheduleDiagnostics(e.document);
        }
    });
    vscode.workspace.onDidOpenTextDocument((doc) => {
        if (doc.languageId === 'link') {
            runDiagnostics(doc);
        }
    });
    vscode.workspace.onDidSaveTextDocument((doc) => {
        if (doc.languageId === 'link') {
            runDiagnostics(doc);
        }
    });
    vscode.workspace.textDocuments.forEach((doc) => {
        if (doc.languageId === 'link') {
            runDiagnostics(doc);
        }
    });
}
function deactivate() {
    if (debounceTimer) {
        clearTimeout(debounceTimer);
    }
}
function scheduleDiagnostics(doc) {
    if (debounceTimer) {
        clearTimeout(debounceTimer);
    }
    debounceTimer = setTimeout(() => {
        runDiagnostics(doc);
    }, 500);
}
function runDiagnostics(doc) {
    const linkCmd = findLinkBinary();
    if (!linkCmd) {
        return;
    }
    const tmpFile = doc.fileName;
    if (!fs.existsSync(tmpFile)) {
        return;
    }
    try {
        const result = child_process.spawnSync(linkCmd, ['run', tmpFile], { encoding: 'utf8', timeout: 5000 });
        const stderr = result.stderr || '';
        const stdout = result.stdout || '';
        const diagnostics = [];
        parseErrors(stderr, diagnostics, doc);
        parseErrors(stdout, diagnostics, doc);
        diagnosticCollection.set(doc.uri, diagnostics);
    }
    catch (e) {
    }
}
function parseErrors(output, diagnostics, doc) {
    const lines = output.split('\n');
    for (const line of lines) {
        const trimmed = line.trim();
        const lexerMatch = trimmed.match(/^Error: .* at line (\d+), col (\d+): (.*)$/);
        if (lexerMatch) {
            const lineNum = parseInt(lexerMatch[1]) - 1;
            const colNum = parseInt(lexerMatch[2]) - 1;
            const message = lexerMatch[3];
            diagnostics.push(makeDiagnostic(doc, lineNum, colNum, message, vscode.DiagnosticSeverity.Error));
            continue;
        }
        const parserMatch = trimmed.match(/^Error.*line (\d+): (.*)$/);
        if (parserMatch) {
            const lineNum = parseInt(parserMatch[1]) - 1;
            const message = parserMatch[2];
            diagnostics.push(makeDiagnostic(doc, lineNum, 0, message, vscode.DiagnosticSeverity.Error));
            continue;
        }
        if (trimmed.startsWith('Error:') && trimmed.includes('line')) {
            const lineMatch = trimmed.match(/line (\d+)/);
            if (lineMatch) {
                const lineNum = parseInt(lineMatch[1]) - 1;
                const message = trimmed.replace(/^Error:\s*/, '');
                diagnostics.push(makeDiagnostic(doc, lineNum, 0, message, vscode.DiagnosticSeverity.Error));
            }
            continue;
        }
        if (trimmed.startsWith('Undefined variable:') ||
            trimmed.startsWith('Cannot') ||
            trimmed.startsWith('Division by zero') ||
            trimmed.startsWith('Modulo by zero') ||
            trimmed.startsWith('Index') ||
            trimmed.startsWith('Expected ')) {
            diagnostics.push(makeDiagnostic(doc, 0, 0, trimmed, vscode.DiagnosticSeverity.Error));
        }
    }
}
function makeDiagnostic(doc, line, col, message, severity) {
    const safeLine = Math.max(0, Math.min(line, doc.lineCount - 1));
    const lineText = doc.lineAt(safeLine).text;
    const safeCol = Math.max(0, Math.min(col, lineText.length));
    let endCol = safeCol + 1;
    if (safeCol < lineText.length) {
        const match = lineText.slice(safeCol).match(/^[a-zA-Z_][a-zA-Z0-9_]*|\S/);
        if (match) {
            endCol = safeCol + match[0].length;
        }
    }
    const range = new vscode.Range(safeLine, safeCol, safeLine, Math.min(endCol, lineText.length));
    return new vscode.Diagnostic(range, message, severity);
}
function findLinkBinary() {
    const candidates = [];
    const workspaceFolders = vscode.workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            candidates.push(path.join(folder.uri.fsPath, 'target', 'debug', 'link.exe'));
            candidates.push(path.join(folder.uri.fsPath, 'target', 'release', 'link.exe'));
        }
    }
    const config = vscode.workspace.getConfiguration('link');
    const customPath = config.get('binaryPath');
    if (customPath) {
        candidates.push(customPath);
    }
    candidates.push('link');
    for (const candidate of candidates) {
        if (path.isAbsolute(candidate) && fs.existsSync(candidate)) {
            return candidate;
        }
        if (!path.isAbsolute(candidate)) {
            try {
                const result = child_process.spawnSync(candidate, ['--version'], { encoding: 'utf8' });
                if (result.status === 0) {
                    return candidate;
                }
            }
            catch (e) {
            }
        }
    }
    return null;
}
//# sourceMappingURL=extension.js.map