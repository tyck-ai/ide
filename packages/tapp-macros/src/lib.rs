use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn::{parse_macro_input, DeriveInput, ItemImpl, Attribute, Meta};

const TAPP_WIT: &str = r#"
package tapp:runtime@0.1.0;

interface app {
    init: func() -> result<_, string>;
    render: func() -> result<string, string>;
    handle: func(action: string) -> result<string, string>;
    shutdown: func() -> result<_, string>;
}

interface tools {
    list-tools: func() -> result<string, string>;
    call-tool: func(name: string, args: string) -> result<string, string>;
}

interface hooks {
    list-hooks: func() -> result<string, string>;
    invoke-hook: func(hook-type: string, data: string) -> result<string, string>;
}

world tapp-app {
    export app;
    export tools;
    export hooks;
}
"#;

#[proc_macro_attribute]
pub fn app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;

    let expanded = quote! {
        #input

        #[cfg(target_arch = "wasm32")]
        mod __tapp_generated {
            use super::*;

            ::wit_bindgen::generate!({
                world: "tapp-app",
                inline: #TAPP_WIT,
            });

            struct TappComponent;

            impl exports::tapp::runtime::app::Guest for TappComponent {
                fn init() -> ::core::result::Result<(), String> {
                    let mut app = #name::default();
                    let ctx = tapp::Context {
                        app_id: std::env::var("TAPP_APP_ID").unwrap_or_else(|_| "unknown".to_string()),
                        data_dir: std::env::var("TAPP_DATA_DIR").unwrap_or_else(|_| ".".to_string()),
                        version: env!("CARGO_PKG_VERSION").to_string(),
                    };
                    <#name as tapp::App>::init(&mut app, &ctx).map_err(|e| e.to_string())?;
                    tapp::__internal::set_app_instance(Box::new(app));
                    Ok(())
                }

                fn render() -> ::core::result::Result<String, String> {
                    use tapp::App;
                    tapp::__internal::reset_id_counter();
                    match tapp::__internal::with_app::<#name, _, _>(|app| app.render()) {
                        Some(tree) => {
                            serde_json::to_string(&tree)
                                .map_err(|e| format!("Failed to serialize UI tree: {}", e))
                        }
                        None => Err("App not initialized".to_string()),
                    }
                }

                fn handle(action: String) -> ::core::result::Result<String, String> {
                    use tapp::App;
                    let action: tapp::Action = serde_json::from_str(&action)
                        .map_err(|e| format!("Failed to parse action: {}", e))?;
                    
                    match tapp::__internal::with_app_mut::<#name, _, _>(|app| app.handle(action)) {
                        Some(Ok(response)) => {
                            serde_json::to_string(&response)
                                .map_err(|e| format!("Failed to serialize response: {}", e))
                        }
                        Some(Err(e)) => Err(e.to_string()),
                        None => Err("App not initialized".to_string()),
                    }
                }

                fn shutdown() -> ::core::result::Result<(), String> {
                    use tapp::App;
                    match tapp::__internal::with_app_mut::<#name, _, _>(|app| app.shutdown()) {
                        Some(Ok(())) => Ok(()),
                        Some(Err(e)) => Err(e.to_string()),
                        None => Err("App not initialized".to_string()),
                    }
                }
            }

            impl exports::tapp::runtime::tools::Guest for TappComponent {
                fn list_tools() -> ::core::result::Result<String, String> {
                    let tools: Vec<tapp::ToolDefinition> = <#name as tapp::TappToolProvider>::__tapp_list_tools();
                    serde_json::to_string(&tools)
                        .map_err(|e| format!("Failed to serialize tools: {}", e))
                }

                fn call_tool(name: String, args: String) -> ::core::result::Result<String, String> {
                    let args: serde_json::Value = serde_json::from_str(&args)
                        .map_err(|e| format!("Failed to parse args: {}", e))?;
                    
                    let result = <#name as tapp::TappToolProvider>::__tapp_call_tool(&name, args);
                    serde_json::to_string(&result)
                        .map_err(|e| format!("Failed to serialize result: {}", e))
                }
            }

            impl exports::tapp::runtime::hooks::Guest for TappComponent {
                fn list_hooks() -> ::core::result::Result<String, String> {
                    let hooks: Vec<tapp::HookRegistration> = <#name as tapp::TappHookProvider>::__tapp_list_hooks();
                    serde_json::to_string(&hooks)
                        .map_err(|e| format!("Failed to serialize hooks: {}", e))
                }

                fn invoke_hook(hook_type: String, data: String) -> ::core::result::Result<String, String> {
                    let data: serde_json::Value = serde_json::from_str(&data)
                        .map_err(|e| format!("Failed to parse data: {}", e))?;
                    
                    let result = <#name as tapp::TappHookProvider>::__tapp_invoke_hook(&hook_type, &data);
                    serde_json::to_string(&result)
                        .map_err(|e| format!("Failed to serialize result: {}", e))
                }
            }

            export!(TappComponent);
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn tools(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    let mut tool_definitions = Vec::new();
    let mut tool_match_arms = Vec::new();

    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let tool_attr = method.attrs.iter().find(|a| a.path().is_ident("tool"));
            if let Some(attr) = tool_attr {
                let method_name = &method.sig.ident;
                let (tool_name, description) = parse_tool_attr(attr);
                let tool_name = tool_name.unwrap_or_else(|| method_name.to_string());
                let description = description.unwrap_or_default();
                let handler = format!("__tapp_tool_{}", method_name);

                tool_definitions.push(quote! {
                    tapp::ToolDefinition {
                        name: #tool_name.to_string(),
                        description: #description.to_string(),
                        handler: #handler.to_string(),
                    }
                });

                tool_match_arms.push(quote! {
                    #tool_name => {
                        match tapp::__internal::with_app_mut::<#self_ty, _, _>(|app| {
                            app.#method_name(args.clone())
                        }) {
                            Some(result) => result,
                            None => tapp::ToolResult::error("App not initialized"),
                        }
                    }
                });
            }
        }
    }

    let expanded = quote! {
        #input

        impl tapp::TappToolProvider for #self_ty {
            fn __tapp_list_tools() -> Vec<tapp::ToolDefinition> {
                vec![#(#tool_definitions),*]
            }

            fn __tapp_call_tool(name: &str, args: serde_json::Value) -> tapp::ToolResult {
                match name {
                    #(#tool_match_arms)*
                    _ => tapp::ToolResult::error(format!("Unknown tool: {}", name)),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_tool_attr(attr: &Attribute) -> (Option<String>, Option<String>) {
    let mut name = None;
    let mut description = None;

    if let Meta::List(meta_list) = &attr.meta {
        let tokens = meta_list.tokens.to_string();
        // Split on commas that are outside of quoted strings
        for (key, value) in split_kv_pairs(&tokens) {
            match key.as_str() {
                "name" => name = Some(value),
                "description" => description = Some(value),
                _ => {}
            }
        }
    }

    (name, description)
}

/// Split `key = "value", key2 = "value2"` respecting quoted strings.
fn split_kv_pairs(input: &str) -> Vec<(String, String)> {
    let mut result = Vec::new();
    let mut chars = input.chars().peekable();

    while chars.peek().is_some() {
        // Skip whitespace and commas
        while chars.peek().map(|c| *c == ',' || c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }

        // Read key
        let key: String = chars.by_ref()
            .take_while(|c| *c != '=' && *c != ',' && !c.is_whitespace())
            .collect();
        if key.is_empty() { break; }

        // Skip whitespace and '='
        while chars.peek().map(|c| *c == '=' || c.is_whitespace()).unwrap_or(false) {
            chars.next();
        }

        // Read value (quoted string)
        let value = if chars.peek() == Some(&'"') {
            chars.next(); // skip opening quote
            let mut val = String::new();
            loop {
                match chars.next() {
                    Some('\\') => { if let Some(c) = chars.next() { val.push(c); } }
                    Some('"') => break,
                    Some(c) => val.push(c),
                    None => break,
                }
            }
            val
        } else {
            // Unquoted value — read until comma or end
            chars.by_ref().take_while(|c| *c != ',').collect::<String>().trim().to_string()
        };

        result.push((key.trim().to_string(), value));
    }
    result
}

#[proc_macro_attribute]
pub fn hooks(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemImpl);
    let self_ty = &input.self_ty;

    let mut hook_registrations = Vec::new();
    let mut hook_match_arms = Vec::new();

    for item in &input.items {
        if let syn::ImplItem::Fn(method) = item {
            let hook_attr = method.attrs.iter().find(|a| a.path().is_ident("hook"));
            if let Some(attr) = hook_attr {
                let method_name = &method.sig.ident;
                let hook_type = parse_hook_attr(attr);
                let handler = format!("__tapp_hook_{}", method_name);

                if let Some(ht) = hook_type {
                    let hook_type_str = match ht.to_string().as_str() {
                        "BeforeInput" => "before_input",
                        "AfterOutput" => "after_output",
                        "OnToolCall" => "on_tool_call",
                        "SessionStart" => "session_start",
                        "SessionEnd" => "session_end",
                        _ => "unknown",
                    };

                    hook_registrations.push(quote! {
                        tapp::HookRegistration {
                            hook_type: tapp::HookType::#ht,
                            handler: #handler.to_string(),
                        }
                    });

                    hook_match_arms.push(quote! {
                        #hook_type_str => {
                            match tapp::__internal::with_app::<#self_ty, _, _>(|app| {
                                app.#method_name(data)
                            }) {
                                Some(result) => result,
                                None => tapp::HookResult::pass_through(),
                            }
                        }
                    });
                }
            }
        }
    }

    let expanded = quote! {
        #input

        impl tapp::TappHookProvider for #self_ty {
            fn __tapp_list_hooks() -> Vec<tapp::HookRegistration> {
                vec![#(#hook_registrations),*]
            }

            fn __tapp_invoke_hook(hook_type: &str, data: &serde_json::Value) -> tapp::HookResult {
                match hook_type {
                    #(#hook_match_arms)*
                    _ => tapp::HookResult::pass_through(),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn parse_hook_attr(attr: &Attribute) -> Option<syn::Ident> {
    if let Meta::List(meta_list) = &attr.meta {
        let tokens = meta_list.tokens.to_string();
        let hook_type = tokens.trim();
        match hook_type {
            "on_before_input" => return Some(format_ident!("BeforeInput")),
            "on_after_output" => return Some(format_ident!("AfterOutput")),
            "on_tool_call" => return Some(format_ident!("OnToolCall")),
            "on_session_start" => return Some(format_ident!("SessionStart")),
            "on_session_end" => return Some(format_ident!("SessionEnd")),
            _ => {}
        }
    }
    None
}

#[proc_macro_attribute]
pub fn tool(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn hook(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_derive(TappToolDefault)]
pub fn derive_tapp_tool_default(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl tapp::TappToolProvider for #name {}
    };
    
    TokenStream::from(expanded)
}

#[proc_macro_derive(TappHookDefault)]
pub fn derive_tapp_hook_default(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let name = &input.ident;
    
    let expanded = quote! {
        impl tapp::TappHookProvider for #name {}
    };
    
    TokenStream::from(expanded)
}
