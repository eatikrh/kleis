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
 * - Step-through debugging (DAP)
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
    // Register REPL commands
    context.subscriptions.push(
        commands.registerCommand('kleis.openRepl', () => {
            ReplPanel.createOrShow(context.extensionUri);
        })
    );

    context.subscriptions.push(
        commands.registerCommand('kleis.runSelection', () => {
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

    // Register debug adapter
    const debugAdapterPath = findDebugAdapter(context);
    if (debugAdapterPath) {
        context.subscriptions.push(
            debug.registerDebugAdapterDescriptorFactory('kleis', 
                new KleisDebugAdapterFactory(debugAdapterPath)
            )
        );
        console.log('Kleis debug adapter registered:', debugAdapterPath);
    } else {
        console.log('Kleis debug adapter not found. Debug with: cargo build --release --bin debug');
    }

    // Find the kleis-lsp server
    const serverPath = findServer(context);
    
    if (!serverPath) {
        window.showWarningMessage(
            'Kleis language server (kleis-lsp) not found. ' +
            'Diagnostics and other advanced features will be disabled. ' +
            'Build it with: cargo build --release --bin kleis-lsp'
        );
        // Still activate - REPL can work without LSP
        console.log('Kleis language extension activated (REPL only, no LSP)');
        return;
    }

    // Server options - run the kleis-lsp binary
    const serverExecutable: Executable = {
        command: serverPath,
        options: {
            env: process.env,
        },
    };

    const serverOptions: ServerOptions = {
        run: serverExecutable,
        debug: serverExecutable,
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
 * Find the kleis-lsp server executable
 */
function findServer(context: ExtensionContext): string | undefined {
    const config = workspace.getConfiguration('kleis');
    
    // 1. Check user-configured path
    const configuredPath = config.get<string>('serverPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }

    // 2. Check for bundled server in extension
    const bundledPath = path.join(context.extensionPath, 'server', 'kleis-lsp');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // 3. Check common build locations relative to workspace
    const workspaceFolders = workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            // Check release build
            const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'kleis-lsp');
            if (fs.existsSync(releasePath)) {
                return releasePath;
            }
            // Check debug build
            const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'kleis-lsp');
            if (fs.existsSync(debugPath)) {
                return debugPath;
            }
        }
    }

    // 4. Check if kleis-lsp is in PATH
    const pathEnv = process.env.PATH || '';
    const pathDirs = pathEnv.split(path.delimiter);
    for (const dir of pathDirs) {
        const serverPath = path.join(dir, 'kleis-lsp');
        if (fs.existsSync(serverPath)) {
            return serverPath;
        }
    }

    return undefined;
}

/**
 * Find the kleis debug adapter executable
 */
function findDebugAdapter(context: ExtensionContext): string | undefined {
    const config = workspace.getConfiguration('kleis');
    
    // 1. Check user-configured path
    const configuredPath = config.get<string>('debugAdapterPath');
    if (configuredPath && fs.existsSync(configuredPath)) {
        return configuredPath;
    }

    // 2. Check for bundled adapter in extension
    const bundledPath = path.join(context.extensionPath, 'server', 'kleis-debug');
    if (fs.existsSync(bundledPath)) {
        return bundledPath;
    }

    // 3. Check common build locations relative to workspace
    const workspaceFolders = workspace.workspaceFolders;
    if (workspaceFolders) {
        for (const folder of workspaceFolders) {
            // Check release build (binary is named 'debug')
            const releasePath = path.join(folder.uri.fsPath, 'target', 'release', 'debug');
            if (fs.existsSync(releasePath)) {
                return releasePath;
            }
            // Check debug build
            const debugPath = path.join(folder.uri.fsPath, 'target', 'debug', 'debug');
            if (fs.existsSync(debugPath)) {
                return debugPath;
            }
        }
    }

    // 4. Check if kleis-debug is in PATH
    const pathEnv = process.env.PATH || '';
    const pathDirs = pathEnv.split(path.delimiter);
    for (const dir of pathDirs) {
        const adapterPath = path.join(dir, 'kleis-debug');
        if (fs.existsSync(adapterPath)) {
            return adapterPath;
        }
    }

    return undefined;
}

/**
 * Debug adapter factory for Kleis
 */
class KleisDebugAdapterFactory implements vscode.DebugAdapterDescriptorFactory {
    private adapterPath: string;

    constructor(adapterPath: string) {
        this.adapterPath = adapterPath;
    }

    createDebugAdapterDescriptor(
        _session: vscode.DebugSession,
        _executable: vscode.DebugAdapterExecutable | undefined
    ): vscode.ProviderResult<vscode.DebugAdapterDescriptor> {
        // Run the debug adapter as an executable
        return new vscode.DebugAdapterExecutable(this.adapterPath, []);
    }
}

