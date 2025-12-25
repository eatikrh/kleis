/**
 * Kleis Language Extension for VS Code
 * 
 * This extension provides:
 * - Syntax highlighting (via TextMate grammar)
 * - Real-time diagnostics (parse errors)
 * - Hover information (type signatures)
 * - Go to definition
 * - Document symbols (outline view)
 * - Interactive REPL panel
 * - Step-through debugging (DAP via unified server)
 * 
 * Architecture:
 * - The unified `kleis server` binary handles LSP over stdio
 * - When debugging starts, we call `kleis.startDebugSession` command
 * - The server spawns DAP on a dynamic TCP port and returns the port
 * - VS Code connects to DAP via TCP
 * - This allows shared state between LSP, DAP, and REPL
 */

import * as path from 'path';
import * as fs from 'fs';
import * as vscode from 'vscode';
import { workspace, ExtensionContext, window, commands, debug } from 'vscode';

import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    Executable,
} from 'vscode-languageclient/node';

import { ReplPanel } from './replPanel';

let client: LanguageClient | undefined;

export function activate(context: ExtensionContext) {
    // Utility: check whether a REPL executable is available
    function replAvailable(): boolean {
        const config = workspace.getConfiguration('kleis');
        const configuredPath = config.get<string>('replPath');
        if (configuredPath && fs.existsSync(configuredPath)) {
            return true;
        }

        const workspaceFolders = workspace.workspaceFolders;
        if (workspaceFolders) {
            for (const folder of workspaceFolders) {
                const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'repl');
                if (fs.existsSync(releasePath)) return true;
                const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'repl');
                if (fs.existsSync(debugPath)) return true;
            }
        }

        const pathEnv = process.env.PATH || '';
        const pathDirs = pathEnv.split(path.delimiter);
        for (const dir of pathDirs) {
            const replPath = path.join(dir, 'repl');
            if (fs.existsSync(replPath)) return true;
        }

        return false;
    }

    // Utility: check whether Kleis server executable is available
    function serverAvailable(): boolean {
        const config = workspace.getConfiguration('kleis');
        const configuredPath = config.get<string>('serverPath');
        if (configuredPath && fs.existsSync(configuredPath)) {
            return true;
        }

        const workspaceFolders = workspace.workspaceFolders;
        if (workspaceFolders) {
            for (const folder of workspaceFolders) {
                const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis');
                if (fs.existsSync(releasePath)) return true;
                const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'kleis');
                if (fs.existsSync(debugPath)) return true;
                // Legacy binary name
                const legacyPath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis-lsp');
                if (fs.existsSync(legacyPath)) return true;
            }
        }

        const pathEnv = process.env.PATH || '';
        const pathDirs = pathEnv.split(path.delimiter);
        for (const dir of pathDirs) {
            const serverPath = path.join(dir, 'kleis');
            if (fs.existsSync(serverPath)) return true;
            const legacyServer = path.join(dir, 'kleis-lsp');
            if (fs.existsSync(legacyServer)) return true;
        }

        return false;
    }

    // Register REPL commands with a quick availability check and user-friendly errors
    context.subscriptions.push(
        commands.registerCommand('kleis.openRepl', () => {
            if (!replAvailable()) {
                window.showErrorMessage(
                    'Kleis REPL not found. Build it with `cargo build --release --bin repl` or set "kleis.replPath" in settings.'
                );
                return;
            }
            ReplPanel.createOrShow(context.extensionUri);
        })
    );

    context.subscriptions.push(
        commands.registerCommand('kleis.runSelection', () => {
            if (!replAvailable()) {
                window.showErrorMessage(
                    'Kleis REPL not found. Build it with `cargo build --release --bin repl` or set "kleis.replPath" in settings.'
                );
                return;
            }

            const editor = window.activeTextEditor;
            if (editor && ReplPanel.currentPanel) {
                const selection = editor.document.getText(editor.selection);
                if (selection) {
                    ReplPanel.currentPanel.sendCommand(selection);
                }
            } else if (!ReplPanel.currentPanel) {
                // Open REPL first, then run selection
                ReplPanel.createOrShow(context.extensionUri);
                setTimeout(() => {
                    const editor = window.activeTextEditor;
                    if (editor && ReplPanel.currentPanel) {
                        const selection = editor.document.getText(editor.selection);
                        if (selection) {
                            ReplPanel.currentPanel.sendCommand(selection);
                        }
                    }
                }, 1000); // Wait for REPL to start
            }
        })
    );

    context.subscriptions.push(
        commands.registerCommand('kleis.loadFileInRepl', () => {
            if (!replAvailable()) {
                window.showErrorMessage(
                    'Kleis REPL not found. Build it with `cargo build --release --bin repl` or set "kleis.replPath" in settings.'
                );
                return;
            }

            const editor = window.activeTextEditor;
            if (editor) {
                const filePath = editor.document.uri.fsPath;
                if (!ReplPanel.currentPanel) {
                    ReplPanel.createOrShow(context.extensionUri);
                    setTimeout(() => {
                        ReplPanel.currentPanel?.loadFile(filePath);
                    }, 1000); // Wait for REPL to start
                } else {
                    ReplPanel.currentPanel.loadFile(filePath);
                }
            }
        })
    );

    // Register debug adapter factory
    // The factory will request a debug port from the unified server via LSP
    context.subscriptions.push(
        debug.registerDebugAdapterDescriptorFactory('kleis', 
            new KleisDebugAdapterFactory()
        )
    );
    console.log('Kleis debug adapter factory registered (uses unified server)');

    // Status command to show LSP/REPL availability in an output channel
    context.subscriptions.push(
        commands.registerCommand('kleis.showStatus', () => {
            const config = workspace.getConfiguration('kleis');
            const serverConfigured = config.get<string>('serverPath') || '(not set)';
            const replConfigured = config.get<string>('replPath') || '(not set)';
            const serverFound = serverAvailable();
            const replFound = replAvailable();

            const out = window.createOutputChannel('Kleis');
            out.clear();
            out.appendLine('Kleis status:');
            out.appendLine(`  Server: ${serverFound ? 'FOUND' : 'MISSING'} (${serverConfigured})`);
            out.appendLine(`  REPL:   ${replFound ? 'FOUND' : 'MISSING'} (${replConfigured})`);
            out.show(true);

            window.showInformationMessage(
                `Kleis status — server: ${serverFound ? 'FOUND' : 'MISSING'}, repl: ${replFound ? 'FOUND' : 'MISSING'}`,
                'Open Output'
            ).then(sel => {
                if (sel === 'Open Output') out.show(true);
            });
        })
    );

    // Find the unified kleis server
    const serverPath = findServer(context);
    
    if (!serverPath) {
        window.showWarningMessage(
            'Kleis unified server not found. ' +
            'Diagnostics, debugging, and other advanced features will be disabled. ' +
            'Build it with: cargo build --release --bin kleis'
        );
        // Still activate - REPL can work without LSP
        console.log('Kleis language extension activated (REPL only, no LSP)');
        return;
    }

    // Respect user trace setting and propagate it to server/LanguageClient
    const traceSetting = workspace.getConfiguration('kleis').get<string>('trace.server') || 'off';

    // Server options - run the unified kleis binary with 'server' subcommand
    const runArgs = ['server'];
    if (traceSetting === 'verbose') runArgs.push('--verbose');

    const serverExecutable: Executable = {
        command: serverPath,
        args: runArgs,
        options: {
            env: process.env,
        },
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        // debug always enables verbose for extra diagnostics
        debug: { ...serverExecutable, args: ['server', '--verbose'] },
    };

    // Client options
    const clientOptions: LanguageClientOptions = {
        // Register for Kleis files
        documentSelector: [{ scheme: 'file', language: 'kleis' }],
        synchronize: {
            // Notify server about file changes to .kleis files
            fileEvents: workspace.createFileSystemWatcher('**/*.kleis'),
        },
    };

    // Create the language client
    client = new LanguageClient(
        'kleis',
        'Kleis Language Server',
        serverOptions,
        clientOptions
    );

    // Set language client trace level so LSP messages appear in the "Language Server" output
    try {
        // Lazy import of Trace enum
        // eslint-disable-next-line @typescript-eslint/no-var-requires
        const { Trace } = require('vscode-languageclient');
        if (traceSetting === 'messages') {
            (client as any).trace = Trace.Messages;
        } else if (traceSetting === 'verbose') {
            (client as any).trace = Trace.Verbose;
        }
    } catch (e) {
        console.warn('Could not set language client trace:', e);
    }

    // Start the client (this also starts the server)
    client.start();

    console.log('Kleis language extension activated with LSP and REPL support');
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}

/**
 * Find the unified kleis server executable
 */
function findServer(context: ExtensionContext): string | undefined {
    const config = workspace.getConfiguration('kleis');
    
    // 1. Check user-configured path
    const configuredPath = config.get<string>('serverPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }

    // 2. Check for bundled server in extension
    const bundledPath = path.join(context.extensionPath, 'server', 'kleis');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // 3. Check common build locations relative to workspace
    const workspaceFolders = workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            // Check release build
            const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis');
            if (fs.existsSync(releasePath)) {
                return releasePath;
            }
            // Check debug build
            const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'kleis');
            if (fs.existsSync(debugPath)) {
                return debugPath;
            }
        }
    }

    // 4. Check if kleis is in PATH
    const pathEnv = process.env.PATH || '';
    const pathDirs = pathEnv.split(path.delimiter);
    for (const dir of pathDirs) {
        const serverPath = path.join(dir, 'kleis');
        if (fs.existsSync(serverPath)) {
            return serverPath;
        }
    }

    // 5. Fallback: check for old kleis-lsp binary for backwards compat
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis-lsp');
            if (fs.existsSync(releasePath)) {
                console.log('Using legacy kleis-lsp binary (consider switching to unified kleis server)');
                return releasePath;
            }
        }
    }

    return undefined;
}

/**
 * Debug adapter factory for Kleis
 * 
 * This factory requests a debug port from the unified server via LSP command,
 * then connects to the DAP server via TCP on that port.
 */
class KleisDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
    async createDebugAdapterDescriptor(
        session: vscode.DebugSession,
        _executable: vscode.DebugAdapterExecutable | undefined
    ): Promise<vscode.DebugAdapterDescriptor> {
        // Get the program path from the debug configuration
        const program = session.configuration.program;
        
        // If we have an LSP client, ask it to start DAP server
        if (client) {
            try {
                console.log('Sending startDebugSession command for program:', program);
                
                // Send command to LSP server to start DAP
                const result = await client.sendRequest('workspace/executeCommand', {
                    command: 'kleis.startDebugSession',
                    arguments: [program]
                }) as { port?: number; error?: string };
                
                console.log('startDebugSession result:', JSON.stringify(result));

                if (result && result.port) {
                    console.log(`Kleis DAP server running on port ${result.port}`);
                    // Connect to DAP via TCP
                    return new vscode.DebugAdapterServer(result.port, '127.0.0.1');
                } else if (result && result.error) {
                    throw new Error(result.error);
                } else {
                    throw new Error('No port returned from kleis.startDebugSession');
                }
            } catch (e) {
                console.error('Failed to start debug session via LSP:', e);
                window.showErrorMessage(`Failed to start Kleis debugger: ${e}`);
                throw e;
            }
        } else {
            // No LSP client available - check for server or standalone adapter and show helpful errors
            const standalonePath = findStandaloneDebugAdapter();
            if (standalonePath) {
                console.log('Using standalone debug adapter (no shared state with LSP)');
                window.showWarningMessage('Kleis LSP server not found; using standalone debug adapter (no shared state with LSP).');
                // Run kleis with 'dap' subcommand for standalone DAP server over stdio
                return new vscode.DebugAdapterExecutable(standalonePath, ['dap']);
            } else {
                window.showErrorMessage('Kleis LSP not found. Debugging requires the Kleis server. Build it with `cargo build --release --bin kleis` or set "kleis.serverPath" in settings.');
                throw new Error('Kleis LSP not found');
            }
        }
    }
}

/**
 * Find standalone debug adapter for fallback when LSP is not running
 * Checks:
 * 1. kleis.debugAdapterPath setting
 * 2. kleis binary in workspace target directories
 * 3. kleis binary in PATH
 */
function findStandaloneDebugAdapter(): string | undefined {
    const config = workspace.getConfiguration('kleis');
    
    // Check configured path first
    const configuredPath = config.get<string>('debugAdapterPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }
    
    // Check workspace build directories for kleis binary
    const workspaceFolders = workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            // Check for unified kleis binary
            const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis');
            if (fs.existsSync(releasePath)) {
                return releasePath;
            }
            const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'kleis');
            if (fs.existsSync(debugPath)) {
                return debugPath;
            }
        }
    }
    
    // Check PATH
    const pathEnv = process.env.PATH || '';
    const pathDirs = pathEnv.split(path.delimiter);
    for (const dir of pathDirs) {
        const kleisPath = path.join(dir, 'kleis');
        if (fs.existsSync(kleisPath)) {
            return kleisPath;
        }
    }
    
    return undefined;
}

