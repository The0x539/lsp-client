use lsp_types::{ClientCapabilities, MarkupKind};
use serde_json::json;

pub fn caps() -> ClientCapabilities {
    let formats = vec![MarkupKind::Markdown, MarkupKind::PlainText];
    let completion_item_kinds = (1..=25).collect::<Vec<_>>();
    let document_symbol_kinds = (1..=26).collect::<Vec<_>>();
    let no_dyn_reg = json!({ "dynamicRegistration": false });

    let capabilities = json!({
        "textDocument": {
            "synchronization": {
                "dynamicRegistration": false,
                "willSave": false,
                "willSaveWaitUntil": false,
                "didSave": true,
            },
            "codeAction": {
                "dynamicRegistration": false,
                "codeActionLiteralSupport": {
                    "codeActionKind": {
                        "valueSet": [
                            "", "quickfix", "refactor",
                            "refactor.extract", "refactor.inline", "refactor.rewrite",
                            "source", "source.organizeImports",
                        ],
                    },
                },
            },
            "completion": {
                "dynamicRegistration": false,
                "completionItem": {
                    "snippetSupport": false,
                    "commitCharatersSupport": false,
                    "preselectSupport": false,
                    "deprecatedSupport": false,
                    "documentationFormat": formats,
                },
                "completionItemKind": { "valueSet": completion_item_kinds },
                "contextSupport": false,
            },
            "declaration": { "linkSupport": true },
            "definition": { "linkSupport": true },
            "implementation": { "linkSupport": true },
            "typeDefinition": { "linkSupport": true },
            "hover": {
                "dynamicRegistration": false,
                "contentFormat": formats,
            },
            "signatureHelp": {
                "dynamicRegistration": false,
                "signatureInformation": {
                    "documentationFormat": formats,
                },
            },
            "references": no_dyn_reg,
            "documentHighlight": no_dyn_reg,
            "documentSymbol": {
                "dynamicRegistration": false,
                "symbolKind": { "valueSet": document_symbol_kinds },
                "hierarchicalDocumentSymbolSupport": true,
            },
            "rename": {
                "dynamicRegistration": false,
                "prepareSupport:": true,
            },
        },
        "workspace": {
            "symbol": {
                "dynamicRegistration": false,
                "symbolKind": { "valueSet": document_symbol_kinds },
                "hierarchicalWorkspaceSymbolSupport": true,
            },
            "workspaceFolders": true,
            "applyEdit": true,
        },
        "callHierarchy": no_dyn_reg,
        "experimental": null,
        "window": { "workDoneProgress": true },
    });

    serde_json::from_value(capabilities).unwrap()
}
