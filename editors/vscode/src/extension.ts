import * as path from 'path';
import * as vscode from 'vscode';
import {
    LanguageClient,
    LanguageClientOptions,
    ServerOptions,
    TransportKind
} from 'vscode-languageclient/node';

let client: LanguageClient;

export function activate(context: vscode.ExtensionContext) {
    const config = vscode.workspace.getConfiguration('bunzo');
    const compilerPath = config.get<string>('compilerPath') || 'bzc';
    const useCompilerLsp = config.get<boolean>('useCompilerLsp') || false;

    let serverOptions: ServerOptions;

    if (useCompilerLsp) {
        serverOptions = {
            run: { command: compilerPath, args: ['lsp'] },
            debug: { command: compilerPath, args: ['lsp'] }
        };
    } else {
        // The server is implemented in node
        const serverModule = context.asAbsolutePath(path.join('out', 'server.js'));
        serverOptions = {
            run: { module: serverModule, transport: TransportKind.ipc },
            debug: {
                module: serverModule,
                transport: TransportKind.ipc,
            }
        };
    }

    // Options to control the language client
    const clientOptions: LanguageClientOptions = {
        // Register the server for Bunzo documents
        documentSelector: [{ scheme: 'file', language: 'bunzo' }],
        synchronize: {
            // Notify the server about file changes to '.bz' files contained in the workspace
            fileEvents: vscode.workspace.createFileSystemWatcher('**/*.bz')
        }
    };

    // Create the language client and start the client.
    client = new LanguageClient(
        'bunzoLanguageServer',
        'Bunzo Language Server',
        serverOptions,
        clientOptions
    );

    // Start the client. This will also launch the server
    client.start();
}

export function deactivate(): Thenable<void> | undefined {
    if (!client) {
        return undefined;
    }
    return client.stop();
}
