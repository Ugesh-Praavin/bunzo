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
                triggerCharacters: ['.', 'a', 'b', 'c', 'd', 'e', 'f', 'g', 'h', 'i', 'j', 'k', 'l', 'm', 'n', 'o', 'p', 'q', 'r', 's', 't', 'u', 'v', 'w', 'x', 'y', 'z', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'S', 'T', 'U', 'V', 'W', 'X', 'Y', 'Z', '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', '_']
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
    // Extract the prefix the user has typed so far (word before cursor)
    let prefix = '';
    let i = offset - 1;
    while (i >= 0 && /[a-zA-Z0-9_]/.test(text[i])) {
        prefix = text[i] + prefix;
        i--;
    }
    // Determine if there is a dot prefix (e.g. math., http., etc.)
    let lastWord = '';
    if (i >= 0 && text[i] === '.') {
        let start = i - 1;
        while (start >= 0 && /[a-zA-Z0-9_]/.test(text[start])) {
            start--;
        }
        lastWord = text.substring(start + 1, i);
    }
    // Standard library dot completion
    if (lastWord && lastWord in stdlibData_1.MODULE_FUNCTIONS) {
        return stdlibData_1.MODULE_FUNCTIONS[lastWord]
            .filter(fn => !prefix || fn.label.startsWith(prefix))
            .map(fn => ({
            label: fn.label,
            kind: fn.kind,
            documentation: fn.documentation
        }));
    }
    // Standard completions (keywords, builtins, types, packages)
    const keywords = [
        'let', 'const', 'var', 'func', 'class', 'struct', 'interface',
        'import', 'export', 'from', 'if', 'else', 'while', 'for', 'in',
        'break', 'continue', 'return', 'try', 'catch', 'throw',
        'extends', 'implements', 'super', 'self', 'enum', 'match', 'switch',
        'move', 'abstract', 'public', 'private', 'trait',
        'spawn', 'async', 'await', 'channel'
    ];
    const builtins = ['print', 'true', 'false', 'null'];
    const types = ['int', 'float', 'string', 'bool', 'any', 'void'];
    const items = [];
    keywords.forEach(keyword => {
        if (!prefix || keyword.startsWith(prefix)) {
            items.push({ label: keyword, kind: node_1.CompletionItemKind.Keyword, data: `keyword_${keyword}` });
        }
    });
    builtins.forEach(builtin => {
        if (!prefix || builtin.startsWith(prefix)) {
            items.push({ label: builtin, kind: node_1.CompletionItemKind.Value, data: `builtin_${builtin}` });
        }
    });
    types.forEach(type => {
        if (!prefix || type.startsWith(prefix)) {
            items.push({ label: type, kind: node_1.CompletionItemKind.TypeParameter, data: `type_${type}` });
        }
    });
    // Add all standard library modules as auto-completions
    stdlibData_1.STDLIB_MODULES.forEach(mod => {
        if (!prefix || mod.startsWith(prefix)) {
            items.push({ label: mod, kind: node_1.CompletionItemKind.Module, data: `module_${mod}` });
        }
    });
    // Snippets (only show if no prefix or matching)
    const snippets = [
        {
            label: 'func template',
            kind: node_1.CompletionItemKind.Snippet,
            insertText: 'func ${1:name}(${2:params}) {\n\t$0\n}',
            insertTextFormat: 2,
            documentation: 'Defines a Bunzo function.'
        },
        {
            label: 'if template',
            kind: node_1.CompletionItemKind.Snippet,
            insertText: 'if ${1:condition} {\n\t$0\n}',
            insertTextFormat: 2,
            documentation: 'If statement.'
        },
        {
            label: 'while template',
            kind: node_1.CompletionItemKind.Snippet,
            insertText: 'while ${1:condition} {\n\t$0\n}',
            insertTextFormat: 2,
            documentation: 'While loop.'
        },
        {
            label: 'for template',
            kind: node_1.CompletionItemKind.Snippet,
            insertText: 'for ${1:i} in ${2:start}..${3:end} {\n\t$0\n}',
            insertTextFormat: 2,
            documentation: 'For loop.'
        }
    ];
    snippets.forEach(s => {
        const insertStr = typeof s.insertText === 'string' ? s.insertText : '';
        if (!prefix || s.label.startsWith(prefix) || insertStr.startsWith(prefix)) {
            items.push(s);
        }
    });
    return items;
});
connection.onCompletionResolve((item) => {
    const docMap = {
        'keyword_func': { detail: 'Function Declaration', documentation: 'Declares a reusable function block.' },
        'keyword_let': { detail: 'Mutable Variable', documentation: 'Declares a mutable block-scoped variable.' },
        'keyword_const': { detail: 'Immutable Constant', documentation: 'Declares a read-only block-scoped constant.' },
        'keyword_var': { detail: 'Mutable Variable', documentation: 'Declares a mutable variable (alias for let).' },
        'keyword_if': { detail: 'Conditional Branch', documentation: 'Executes a block if a condition is true.' },
        'keyword_else': { detail: 'Alternate Branch', documentation: 'Executes a block when the preceding if condition is false.' },
        'keyword_while': { detail: 'While Loop', documentation: 'Repeats a block while a condition is true.' },
        'keyword_for': { detail: 'For Loop', documentation: 'Iterates over a range or collection.' },
        'keyword_in': { detail: 'Range Keyword', documentation: 'Used in for-in range expressions.' },
        'keyword_break': { detail: 'Break Statement', documentation: 'Exits the current loop.' },
        'keyword_continue': { detail: 'Continue Statement', documentation: 'Skips to the next loop iteration.' },
        'keyword_return': { detail: 'Return Statement', documentation: 'Returns a value from a function.' },
        'keyword_class': { detail: 'Class Declaration', documentation: 'Declares a class with fields, methods, and inheritance.' },
        'keyword_struct': { detail: 'Struct Declaration', documentation: 'Declares a lightweight value-type data structure.' },
        'keyword_interface': { detail: 'Interface Declaration', documentation: 'Declares a contract that classes can implement.' },
        'keyword_import': { detail: 'Import Statement', documentation: 'Imports a module.' },
        'keyword_export': { detail: 'Export Statement', documentation: 'Exports a symbol from a module.' },
        'keyword_from': { detail: 'From Clause', documentation: 'Specifies the source module in an import.' },
        'keyword_try': { detail: 'Try Block', documentation: 'Attempts an operation that may throw.' },
        'keyword_catch': { detail: 'Catch Block', documentation: 'Catches and handles thrown errors.' },
        'keyword_throw': { detail: 'Throw Statement', documentation: 'Throws an error value.' },
        'keyword_extends': { detail: 'Inheritance', documentation: 'Specifies a parent class for inheritance.' },
        'keyword_implements': { detail: 'Interface Implementation', documentation: 'Declares that a class implements an interface.' },
        'keyword_super': { detail: 'Parent Reference', documentation: 'References the parent class.' },
        'keyword_self': { detail: 'Self Reference', documentation: 'References the current instance.' },
        'keyword_enum': { detail: 'Enum Declaration', documentation: 'Declares an enumerated type.' },
        'keyword_match': { detail: 'Pattern Matching', documentation: 'Performs pattern matching on a value.' },
        'keyword_switch': { detail: 'Switch Statement', documentation: 'Multi-branch conditional (reserved).' },
        'keyword_move': { detail: 'Move Ownership', documentation: 'Transfers ownership of a value.' },
        'keyword_abstract': { detail: 'Abstract Modifier', documentation: 'Declares an abstract class or method.' },
        'keyword_public': { detail: 'Public Access', documentation: 'Makes a field or method publicly accessible.' },
        'keyword_private': { detail: 'Private Access', documentation: 'Restricts access to the declaring scope.' },
        'keyword_trait': { detail: 'Trait Declaration', documentation: 'Alias for interface, defines shared behavior.' },
        'keyword_spawn': { detail: 'Spawn Task', documentation: 'Spawns a concurrent task.' },
        'keyword_async': { detail: 'Async Modifier', documentation: 'Marks a function as asynchronous.' },
        'keyword_await': { detail: 'Await Expression', documentation: 'Awaits completion of an async operation.' },
        'keyword_channel': { detail: 'Channel', documentation: 'Creates a communication channel.' },
        'builtin_print': { detail: 'Print Statement', documentation: 'Prints the string representation of a value to stdout.' },
        'builtin_true': { detail: 'Boolean True', documentation: 'Boolean literal true.' },
        'builtin_false': { detail: 'Boolean False', documentation: 'Boolean literal false.' },
        'builtin_null': { detail: 'Null Value', documentation: 'Represents the absence of a value.' },
    };
    if (item.data && typeof item.data === 'string') {
        const entry = docMap[item.data];
        if (entry) {
            item.detail = entry.detail;
            item.documentation = entry.documentation;
        }
        if (item.data.startsWith('module_')) {
            const modName = item.data.replace('module_', '');
            item.detail = `Module '${modName}'`;
            item.documentation = `Import '${modName}' to use standard library functions.`;
        }
        if (item.data.startsWith('type_')) {
            const typeName = item.data.replace('type_', '');
            item.detail = `Type '${typeName}'`;
            item.documentation = `Bunzo ${typeName} type.`;
        }
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