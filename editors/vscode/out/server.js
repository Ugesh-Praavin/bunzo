"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const node_1 = require("vscode-languageserver/node");
const vscode_languageserver_textdocument_1 = require("vscode-languageserver-textdocument");
const stdlibData_1 = require("./stdlibData");
// Create a connection for the server, using Node's IPC as a transport.
const connection = (0, node_1.createConnection)(node_1.ProposedFeatures.all);
// Create a simple text document manager.
const documents = new node_1.TextDocuments(vscode_languageserver_textdocument_1.TextDocument);
let hasConfigurationCapability = false;
let hasWorkspaceFolderCapability = false;
connection.onInitialize((params) => {
    const capabilities = params.capabilities;
    hasConfigurationCapability = !!(capabilities.workspace && !!capabilities.workspace.configuration);
    hasWorkspaceFolderCapability = !!(capabilities.workspace && !!capabilities.workspace.workspaceFolders);
    const result = {
        capabilities: {
            textDocumentSync: node_1.TextDocumentSyncKind.Incremental,
            // Enable completion.
            completionProvider: {
                resolveProvider: true,
                triggerCharacters: ['.']
            },
            // Enable hover tooltips.
            hoverProvider: true,
            // Enable Signature Help (Parameter Hints) when user types '(' or ','.
            signatureHelpProvider: {
                triggerCharacters: ['(', ',']
            }
        }
    };
    if (hasWorkspaceFolderCapability) {
        result.capabilities.workspace = {
            workspaceFolders: {
                supported: true
            }
        };
    }
    return result;
});
connection.onInitialized(() => {
    if (hasConfigurationCapability) {
        connection.client.register(node_1.DidChangeConfigurationNotification.type, undefined);
    }
});
const defaultSettings = { compilerPath: 'bzc', useCompilerLsp: false };
let globalSettings = defaultSettings;
const documentSettings = new Map();
connection.onDidChangeConfiguration(change => {
    if (hasConfigurationCapability) {
        documentSettings.clear();
    }
    else {
        globalSettings = ((change.settings.bunzo || defaultSettings));
    }
    documents.all().forEach(validateTextDocument);
});
documents.onDidClose(e => {
    documentSettings.delete(e.document.uri);
});
documents.onDidChangeContent(change => {
    validateTextDocument(change.document);
});
async function validateTextDocument(textDocument) {
    const text = textDocument.getText();
    const diagnostics = [];
    let lines = text.split(/\r?\n/);
    let openBrackets = 0;
    for (let i = 0; i < lines.length; i++) {
        let line = lines[i];
        for (let char of line) {
            if (char === '{')
                openBrackets++;
            if (char === '}')
                openBrackets--;
        }
        // Constant reassignment checking
        let constMatch = line.match(/^\s*const\s+(\w+)\s*=/);
        if (constMatch) {
            let constName = constMatch[1];
            let reassignmentRegex = new RegExp(`^\\s*${constName}\\s*(?:=|\\+=|-=|\\*=|=)`);
            for (let j = i + 1; j < lines.length; j++) {
                if (reassignmentRegex.test(lines[j])) {
                    diagnostics.push({
                        severity: node_1.DiagnosticSeverity.Error,
                        range: {
                            start: { line: j, character: lines[j].indexOf(constName) },
                            end: { line: j, character: lines[j].indexOf(constName) + constName.length }
                        },
                        message: `Cannot reassign constant variable '${constName}'.`,
                        source: 'Bunzo Linter'
                    });
                }
            }
        }
    }
    if (openBrackets !== 0) {
        let lastLine = lines.length - 1;
        diagnostics.push({
            severity: node_1.DiagnosticSeverity.Warning,
            range: {
                start: { line: 0, character: 0 },
                end: { line: lastLine, character: lines[lastLine].length }
            },
            message: `Mismatched brackets: ${Math.abs(openBrackets)} ${openBrackets > 0 ? 'unclosed' : 'extra'} bracket(s).`,
            source: 'Bunzo Parser'
        });
    }
    connection.sendDiagnostics({ uri: textDocument.uri, diagnostics });
}
// Completion provider
connection.onCompletion((textDocumentPosition) => {
    const document = documents.get(textDocumentPosition.textDocument.uri);
    if (!document)
        return [];
    const offset = document.offsetAt(textDocumentPosition.position);
    const text = document.getText();
    // Determine if there is a dot prefix (e.g. math., http., etc.)
    let lastWord = '';
    let i = offset - 1;
    while (i >= 0 && /\s/.test(text[i])) {
        i--;
    }
    if (i >= 0 && text[i] === '.') {
        let start = i - 1;
        while (start >= 0 && /[a-zA-Z0-9_]/.test(text[start])) {
            start--;
        }
        lastWord = text.substring(start + 1, i);
    }
    // Standard library dot completion
    if (lastWord && lastWord in stdlibData_1.MODULE_FUNCTIONS) {
        return stdlibData_1.MODULE_FUNCTIONS[lastWord].map(fn => ({
            label: fn.label,
            kind: fn.kind,
            documentation: fn.documentation
        }));
    }
    // Standard completions (keywords, builtins, types, packages)
    const keywords = [
        'let', 'const', 'func', 'class', 'struct', 'interface',
        'import', 'export', 'if', 'else', 'while', 'for', 'in',
        'break', 'continue', 'return', 'try', 'catch', 'throw'
    ];
    const builtins = ['print', 'true', 'false', 'null'];
    const types = ['int', 'float', 'string', 'bool', 'any', 'void'];
    const items = [];
    keywords.forEach(keyword => {
        items.push({ label: keyword, kind: node_1.CompletionItemKind.Keyword, data: `keyword_${keyword}` });
    });
    builtins.forEach(builtin => {
        items.push({ label: builtin, kind: node_1.CompletionItemKind.Value, data: `builtin_${builtin}` });
    });
    types.forEach(type => {
        items.push({ label: type, kind: node_1.CompletionItemKind.TypeParameter, data: `type_${type}` });
    });
    // Add all standard library modules as auto-completions
    stdlibData_1.STDLIB_MODULES.forEach(mod => {
        items.push({ label: mod, kind: node_1.CompletionItemKind.Module, data: `module_${mod}` });
    });
    // Snippets
    items.push({
        label: 'func template',
        kind: node_1.CompletionItemKind.Snippet,
        insertText: 'func ${1:name}(${2:params}) {\n\t$0\n}',
        insertTextFormat: 2,
        documentation: 'Defines a Bunzo function.'
    });
    items.push({
        label: 'if template',
        kind: node_1.CompletionItemKind.Snippet,
        insertText: 'if ${1:condition} {\n\t$0\n}',
        insertTextFormat: 2,
        documentation: 'If statement.'
    });
    items.push({
        label: 'while template',
        kind: node_1.CompletionItemKind.Snippet,
        insertText: 'while ${1:condition} {\n\t$0\n}',
        insertTextFormat: 2,
        documentation: 'While loop.'
    });
    items.push({
        label: 'for template',
        kind: node_1.CompletionItemKind.Snippet,
        insertText: 'for ${1:i} in ${2:start}..${3:end} {\n\t$0\n}',
        insertTextFormat: 2,
        documentation: 'For loop.'
    });
    return items;
});
connection.onCompletionResolve((item) => {
    if (item.data === 'keyword_func') {
        item.detail = 'Function Declaration';
        item.documentation = 'Declares a reusable function block.';
    }
    else if (item.data === 'keyword_let') {
        item.detail = 'Mutable Variable';
        item.documentation = 'Declares a mutable block-scoped variable.';
    }
    else if (item.data === 'keyword_const') {
        item.detail = 'Immutable Constant';
        item.documentation = 'Declares a read-only block-scoped constant.';
    }
    else if (item.data === 'builtin_print') {
        item.detail = 'Print Statement';
        item.documentation = 'Prints the string representation of a value to stdout.';
    }
    else if (item.data && item.data.toString().startsWith('module_')) {
        const modName = item.data.toString().replace('module_', '');
        item.detail = `Module '${modName}'`;
        item.documentation = `Import '${modName}' to use standard library functions.`;
    }
    return item;
});
// Hover Provider
connection.onHover((params) => {
    const document = documents.get(params.textDocument.uri);
    if (!document)
        return null;
    const offset = document.offsetAt(params.position);
    const text = document.getText();
    let word = '';
    let start = offset;
    while (start > 0 && /[a-zA-Z0-9_\.]/.test(text[start - 1])) {
        start--;
    }
    let end = offset;
    while (end < text.length && /[a-zA-Z0-9_\.]/.test(text[end])) {
        end++;
    }
    word = text.substring(start, end);
    const docs = {
        print: '**print(value)**: Built-in function to write values to output.',
        let: '**let**: Declares a mutable variable.',
        const: '**const**: Declares a read-only constant.',
        func: '**func**: Declares a function.',
        class: '**class**: Declares a class, supporting fields, methods, inheritance, and interfaces.',
        struct: '**struct**: Declares a lightweight, value-type data structure.',
        interface: '**interface**: Declares a contract specifying methods classes must implement.'
    };
    // Use HOVER_DOCS if available
    let mdDoc = docs[word] || stdlibData_1.HOVER_DOCS[word];
    if (!mdDoc) {
        const sig = stdlibData_1.SIGNATURES[word];
        if (sig) {
            mdDoc = `**${sig.label}**\n\n${sig.docs}`;
        }
    }
    if (mdDoc) {
        return {
            contents: {
                kind: 'markdown',
                value: mdDoc
            }
        };
    }
    return null;
});
// Signature Help / Parameter Hints provider
connection.onSignatureHelp((params) => {
    const document = documents.get(params.textDocument.uri);
    if (!document)
        return null;
    const offset = document.offsetAt(params.position);
    const text = document.getText();
    let openParenIndex = -1;
    let parenCount = 0;
    for (let i = offset - 1; i >= 0; i--) {
        if (text[i] === ')')
            parenCount++;
        if (text[i] === '(') {
            if (parenCount > 0) {
                parenCount--;
            }
            else {
                openParenIndex = i;
                break;
            }
        }
    }
    if (openParenIndex === -1)
        return null;
    let start = openParenIndex - 1;
    while (start >= 0 && /\s/.test(text[start])) {
        start--;
    }
    let end = start + 1;
    while (start >= 0 && /[a-zA-Z0-9_\.]/.test(text[start])) {
        start--;
    }
    const funcName = text.substring(start + 1, end);
    let paramIndex = 0;
    for (let i = openParenIndex + 1; i < offset; i++) {
        if (text[i] === ',')
            paramIndex++;
    }
    const sig = stdlibData_1.SIGNATURES[funcName];
    if (!sig)
        return null;
    return {
        signatures: [{
                label: sig.label,
                documentation: {
                    kind: 'markdown',
                    value: sig.docs
                },
                parameters: sig.params.map(p => ({
                    label: p.label,
                    documentation: {
                        kind: 'markdown',
                        value: p.docs
                    }
                }))
            }],
        activeSignature: 0,
        activeParameter: Math.min(paramIndex, sig.params.length - 1)
    };
});
documents.listen(connection);
connection.listen();
//# sourceMappingURL=server.js.map