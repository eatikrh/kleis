/**
 * Kleis REPL Panel for VS Code
 * 
 * Provides an interactive webview panel for the Kleis REPL,
 * allowing users to evaluate expressions and see results
 * with rich formatting.
 */

import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';
import { spawn, ChildProcess } from 'child_process';

export class ReplPanel {
    public static currentPanel: ReplPanel | undefined;
    private readonly panel: vscode.WebviewPanel;
    private readonly extensionUri: vscode.Uri;
    private replProcess: ChildProcess | null = null;
    private outputBuffer: string = '';
    private disposables: vscode.Disposable[] = [];

    private constructor(panel: vscode.WebviewPanel, extensionUri: vscode.Uri) {
        this.panel = panel;
        this.extensionUri = extensionUri;

        // Set the webview's initial HTML content
        this.panel.webview.html = this.getHtmlForWebview();

        // Handle messages from the webview
        this.panel.webview.onDidReceiveMessage(
            message => this.handleMessage(message),
            null,
            this.disposables
        );

        // Handle panel disposal
        this.panel.onDidDispose(() => this.dispose(), null, this.disposables);

        // Start the REPL process
        this.startRepl();
    }

    /**
     * Create or show the REPL panel
     */
    public static createOrShow(extensionUri: vscode.Uri) {
        const column = vscode.ViewColumn.Beside;

        // If we already have a panel, show it
        if (ReplPanel.currentPanel) {
            ReplPanel.currentPanel.panel.reveal(column);
            return;
        }

        // Create a new panel
        const panel = vscode.window.createWebviewPanel(
            'kleisRepl',
            'Kleis REPL',
            column,
            {
                enableScripts: true,
                retainContextWhenHidden: true,
                localResourceRoots: [
                    vscode.Uri.joinPath(extensionUri, 'media')
                ]
            }
        );

        ReplPanel.currentPanel = new ReplPanel(panel, extensionUri);
    }

    /**
     * Send a command to the REPL
     */
    public sendCommand(command: string) {
        if (this.replProcess && this.replProcess.stdin) {
            // Echo the command to the webview
            this.sendToWebview({
                type: 'input',
                text: command
            });
            // Send to REPL process
            this.replProcess.stdin.write(command + '\n');
        } else {
            this.sendToWebview({
                type: 'error',
                text: 'REPL process not running. Attempting to restart...'
            });
            this.startRepl();
        }
    }

    /**
     * Load a file in the REPL
     */
    public loadFile(filePath: string) {
        this.sendCommand(`:load ${filePath}`);
    }

    /**
     * Find the kleis-repl executable
     */
    private findRepl(): string | undefined {
        const config = vscode.workspace.getConfiguration('kleis');
        
        // 1. Check user-configured path
        const configuredPath = config.get<string>('replPath');
        if (configuredPath && fs.existsSync(configuredPath)) {
            return configuredPath;
        }

        // 2. Check common build locations relative to workspace
        // The REPL binary is named 'repl' (not 'kleis-repl')
        const workspaceFolders = vscode.workspace.workspaceFolders;
        if (workspaceFolders) {
            for (const folder of workspaceFolders) {
                // Check release build
                const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'repl');
                if (fs.existsSync(releasePath)) {
                    return releasePath;
                }
                // Check debug build
                const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'repl');
                if (fs.existsSync(debugPath)) {
                    return debugPath;
                }
            }
        }

        // 3. Check if repl is in PATH
        const pathEnv = process.env.PATH || '';
        const pathDirs = pathEnv.split(path.delimiter);
        for (const dir of pathDirs) {
            const replPath = path.join(dir, 'repl');
            if (fs.existsSync(replPath)) {
                return replPath;
            }
        }

        return undefined;
    }

    /**
     * Start the REPL process
     */
    private startRepl() {
        const replPath = this.findRepl();
        
        if (!replPath) {
            this.sendToWebview({
                type: 'error',
                text: 'Kleis REPL not found.\n' +
                      'Build it with: cargo build --release --bin repl\n' +
                      'Or set kleis.replPath in settings.'
            });
            return;
        }

        try {
            this.replProcess = spawn(replPath, [], {
                stdio: ['pipe', 'pipe', 'pipe'],
                cwd: vscode.workspace.workspaceFolders?.[0]?.uri.fsPath
            });

            // Handle stdout
            this.replProcess.stdout?.on('data', (data: Buffer) => {
                const text = data.toString();
                this.outputBuffer += text;
                
                // Send complete lines to webview
                const lines = this.outputBuffer.split('\n');
                // Keep the last incomplete line in buffer
                this.outputBuffer = lines.pop() || '';
                
                if (lines.length > 0) {
                    this.sendToWebview({
                        type: 'output',
                        text: lines.join('\n')
                    });
                }
            });

            // Handle stderr
            this.replProcess.stderr?.on('data', (data: Buffer) => {
                this.sendToWebview({
                    type: 'error',
                    text: data.toString()
                });
            });

            // Handle process exit
            this.replProcess.on('close', (code) => {
                this.sendToWebview({
                    type: 'status',
                    text: `REPL process exited with code ${code}`
                });
                this.replProcess = null;
            });

            // Handle process error
            this.replProcess.on('error', (err) => {
                this.sendToWebview({
                    type: 'error',
                    text: `Failed to start REPL: ${err.message}`
                });
                this.replProcess = null;
            });

            this.sendToWebview({
                type: 'status',
                text: `Connected to Kleis REPL (${replPath})`
            });

        } catch (err) {
            this.sendToWebview({
                type: 'error',
                text: `Error starting REPL: ${err}`
            });
        }
    }

    /**
     * Handle messages from the webview
     */
    private handleMessage(message: { type: string; text?: string }) {
        switch (message.type) {
            case 'command':
                if (message.text) {
                    this.sendCommand(message.text);
                }
                break;
            case 'restart':
                if (this.replProcess) {
                    this.replProcess.kill();
                }
                this.startRepl();
                break;
        }
    }

    /**
     * Send a message to the webview
     */
    private sendToWebview(message: { type: string; text: string }) {
        this.panel.webview.postMessage(message);
    }

    /**
     * Get the HTML content for the webview
     */
    private getHtmlForWebview(): string {
        return `<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Kleis REPL</title>
    <style>
        :root {
            --bg-color: var(--vscode-editor-background);
            --text-color: var(--vscode-editor-foreground);
            --input-bg: var(--vscode-input-background);
            --input-border: var(--vscode-input-border);
            --prompt-color: #6A9955;
            --error-color: #F44747;
            --status-color: #569CD6;
            --output-color: var(--vscode-editor-foreground);
        }
        
        body {
            font-family: var(--vscode-editor-font-family, 'Consolas', 'Courier New', monospace);
            font-size: var(--vscode-editor-font-size, 14px);
            background-color: var(--bg-color);
            color: var(--text-color);
            margin: 0;
            padding: 0;
            height: 100vh;
            display: flex;
            flex-direction: column;
        }
        
        #toolbar {
            display: flex;
            align-items: center;
            padding: 8px;
            border-bottom: 1px solid var(--vscode-panel-border);
            background: var(--vscode-sideBar-background);
        }
        
        #toolbar button {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            padding: 4px 12px;
            margin-right: 8px;
            cursor: pointer;
            border-radius: 3px;
        }
        
        #toolbar button:hover {
            background: var(--vscode-button-hoverBackground);
        }
        
        #output {
            flex: 1;
            overflow-y: auto;
            padding: 12px;
            white-space: pre-wrap;
            word-wrap: break-word;
        }
        
        .line {
            margin: 2px 0;
        }
        
        .line.input {
            color: var(--prompt-color);
        }
        
        .line.input::before {
            content: 'λ> ';
            color: var(--prompt-color);
        }
        
        .line.output {
            color: var(--output-color);
        }
        
        .line.error {
            color: var(--error-color);
        }
        
        .line.status {
            color: var(--status-color);
            font-style: italic;
        }
        
        #input-container {
            display: flex;
            align-items: center;
            padding: 8px 12px;
            border-top: 1px solid var(--vscode-panel-border);
            background: var(--vscode-sideBar-background);
        }
        
        #prompt {
            color: var(--prompt-color);
            margin-right: 8px;
            font-weight: bold;
        }
        
        #input {
            flex: 1;
            background: var(--input-bg);
            color: var(--text-color);
            border: 1px solid var(--input-border);
            padding: 6px 10px;
            font-family: inherit;
            font-size: inherit;
            outline: none;
            border-radius: 3px;
        }
        
        #input:focus {
            border-color: var(--vscode-focusBorder);
        }
        
        /* Matrix rendering */
        .matrix {
            display: inline-block;
            margin: 4px 0;
        }
        
        .matrix-bracket {
            font-size: 1.5em;
            vertical-align: middle;
        }
        
        .matrix-content {
            display: inline-grid;
            vertical-align: middle;
            gap: 4px 12px;
        }
    </style>
</head>
<body>
    <div id="toolbar">
        <button id="restart-btn" title="Restart REPL">⟳ Restart</button>
        <button id="clear-btn" title="Clear output">⌫ Clear</button>
        <span id="status">Connecting...</span>
    </div>
    
    <div id="output"></div>
    
    <div id="input-container">
        <span id="prompt">λ></span>
        <input type="text" id="input" placeholder="Enter command..." autofocus>
    </div>

    <script>
        const vscode = acquireVsCodeApi();
        const output = document.getElementById('output');
        const input = document.getElementById('input');
        const statusEl = document.getElementById('status');
        
        // Command history
        let history = [];
        let historyIndex = -1;
        
        // Handle input
        input.addEventListener('keydown', (e) => {
            if (e.key === 'Enter' && input.value.trim()) {
                const command = input.value;
                history.push(command);
                historyIndex = history.length;
                vscode.postMessage({ type: 'command', text: command });
                input.value = '';
            } else if (e.key === 'ArrowUp') {
                if (historyIndex > 0) {
                    historyIndex--;
                    input.value = history[historyIndex];
                }
                e.preventDefault();
            } else if (e.key === 'ArrowDown') {
                if (historyIndex < history.length - 1) {
                    historyIndex++;
                    input.value = history[historyIndex];
                } else {
                    historyIndex = history.length;
                    input.value = '';
                }
                e.preventDefault();
            }
        });
        
        // Handle restart button
        document.getElementById('restart-btn').addEventListener('click', () => {
            vscode.postMessage({ type: 'restart' });
        });
        
        // Handle clear button
        document.getElementById('clear-btn').addEventListener('click', () => {
            output.innerHTML = '';
        });
        
        // Handle messages from extension
        window.addEventListener('message', event => {
            const message = event.data;
            const line = document.createElement('div');
            line.className = 'line ' + message.type;
            
            if (message.type === 'status') {
                statusEl.textContent = message.text;
            }
            
            // Render the text (could add rich rendering here)
            line.textContent = message.text;
            output.appendChild(line);
            
            // Auto-scroll to bottom
            output.scrollTop = output.scrollHeight;
        });
        
        // Focus input on click anywhere
        document.body.addEventListener('click', () => {
            input.focus();
        });
    </script>
</body>
</html>`;
    }

    /**
     * Clean up resources
     */
    public dispose() {
        ReplPanel.currentPanel = undefined;

        // Kill the REPL process
        if (this.replProcess) {
            this.replProcess.kill();
            this.replProcess = null;
        }

        // Dispose panel
        this.panel.dispose();

        // Dispose all disposables
        while (this.disposables.length) {
            const disposable = this.disposables.pop();
            if (disposable) {
                disposable.dispose();
            }
        }
    }
}

