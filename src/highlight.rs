use std::path::Path;
use std::sync::OnceLock;

use syntect::easy::ScopeRegionIterator;
use syntect::parsing::{ParseState, Scope, ScopeStack, SyntaxReference, SyntaxSet};

pub fn highlight_lines<'a, I>(path: &str, lines: I, use_first_line_hint: bool) -> Vec<String>
where
    I: IntoIterator<Item = &'a str>,
{
    let raw_lines: Vec<&str> = lines.into_iter().collect();
    let syntax_set = syntax_set();
    let syntax = syntax_for_path(
        syntax_set,
        path,
        use_first_line_hint
            .then(|| raw_lines.first().copied())
            .flatten(),
    );
    let mut parse_state = ParseState::new(syntax);
    let mut scope_stack = ScopeStack::new();
    let mut highlighted = Vec::with_capacity(raw_lines.len());

    for line in &raw_lines {
        let Ok(ops) = parse_state.parse_line(line, syntax_set) else {
            return escape_lines(raw_lines);
        };

        let mut html = String::with_capacity(line.len() + 16);

        for (segment, op) in ScopeRegionIterator::new(&ops, line) {
            if scope_stack.apply(&op).is_err() {
                return escape_lines(raw_lines);
            }

            if segment.is_empty() {
                continue;
            }

            if let Some(class) = token_class(scope_stack.as_slice()) {
                html.push_str("<span class=\"");
                html.push_str(class);
                html.push_str("\">");
                push_escaped(&mut html, segment);
                html.push_str("</span>");
            } else {
                push_escaped(&mut html, segment);
            }
        }

        highlighted.push(html);
    }

    highlighted
}

fn syntax_set() -> &'static SyntaxSet {
    static SYNTAX_SET: OnceLock<SyntaxSet> = OnceLock::new();
    SYNTAX_SET.get_or_init(SyntaxSet::load_defaults_nonewlines)
}

fn syntax_for_path<'a>(
    syntax_set: &'a SyntaxSet,
    path: &str,
    first_line: Option<&str>,
) -> &'a SyntaxReference {
    let file_name = Path::new(path).file_name().and_then(|name| name.to_str());

    file_name
        .and_then(|name| syntax_set.find_syntax_by_token(name))
        .or_else(|| {
            Path::new(path)
                .extension()
                .and_then(|ext| ext.to_str())
                .and_then(|ext| syntax_set.find_syntax_by_token(ext))
        })
        .or_else(|| first_line.and_then(|line| syntax_set.find_syntax_by_first_line(line)))
        .unwrap_or_else(|| syntax_set.find_syntax_plain_text())
}

fn token_class(stack: &[Scope]) -> Option<&'static str> {
    let scopes = token_scopes();

    if matches_scope(stack, &[scopes.comment]) {
        Some("tok-comment")
    } else if matches_scope(
        stack,
        &[scopes.string, scopes.string_escape, scopes.string_regexp],
    ) {
        Some("tok-string")
    } else if matches_scope(stack, &[scopes.number]) {
        Some("tok-number")
    } else if matches_scope(stack, &[scopes.attribute_name]) {
        Some("tok-attribute")
    } else if matches_scope(stack, &[scopes.decorator]) {
        Some("tok-decorator")
    } else if matches_scope(
        stack,
        &[
            scopes.namespace,
            scopes.support_namespace,
            scopes.entity_type,
            scopes.support_type,
            scopes.storage_type,
            scopes.storage_modifier,
            scopes.entity_class,
            scopes.entity_struct,
            scopes.entity_enum,
            scopes.entity_union,
            scopes.entity_interface,
            scopes.entity_trait,
        ],
    ) {
        Some("tok-type")
    } else if matches_scope(
        stack,
        &[scopes.macro_name, scopes.entity_macro, scopes.preprocessor],
    ) {
        Some("tok-macro")
    } else if matches_scope(
        stack,
        &[
            scopes.entity_function,
            scopes.constructor,
            scopes.variable_function,
            scopes.support_function,
            scopes.builtin_function,
            scopes.builtin_variable,
            scopes.builtin_constant,
        ],
    ) {
        Some("tok-function")
    } else if matches_scope(
        stack,
        &[
            scopes.constant,
            scopes.support_constant,
            scopes.constant_character,
            scopes.variable_parameter,
            scopes.variable_property,
            scopes.variable_member,
            scopes.entity_field,
        ],
    ) {
        Some("tok-constant")
    } else if matches_scope(stack, &[scopes.keyword_control]) {
        Some("tok-keyword-control")
    } else if matches_scope(
        stack,
        &[
            scopes.keyword,
            scopes.bool,
            scopes.null,
            scopes.type_builtin,
            scopes.support_type_builtin,
            scopes.support_class_builtin,
            scopes.entity_tag,
            scopes.variable_language,
            scopes.keyword_operator,
            scopes.punctuation,
            scopes.punctuation_definition,
            scopes.punctuation_separator,
            scopes.punctuation_terminator,
            scopes.punctuation_section,
            scopes.punctuation_accessor,
        ],
    ) {
        Some("tok-keyword")
    } else if matches_scope(stack, &[scopes.variable]) {
        Some("tok-variable")
    } else {
        None
    }
}

fn matches_scope(stack: &[Scope], prefixes: &[Scope]) -> bool {
    stack
        .iter()
        .any(|scope| prefixes.iter().any(|prefix| prefix.is_prefix_of(*scope)))
}

fn token_scopes() -> &'static TokenScopes {
    static TOKEN_SCOPES: OnceLock<TokenScopes> = OnceLock::new();
    TOKEN_SCOPES.get_or_init(TokenScopes::new)
}

struct TokenScopes {
    comment: Scope,
    string: Scope,
    string_escape: Scope,
    string_regexp: Scope,
    number: Scope,
    bool: Scope,
    null: Scope,
    keyword: Scope,
    keyword_control: Scope,
    keyword_operator: Scope,
    storage_modifier: Scope,
    storage_type: Scope,
    type_builtin: Scope,
    entity_tag: Scope,
    entity_type: Scope,
    entity_class: Scope,
    entity_struct: Scope,
    entity_enum: Scope,
    entity_union: Scope,
    entity_interface: Scope,
    entity_trait: Scope,
    namespace: Scope,
    attribute_name: Scope,
    decorator: Scope,
    entity_function: Scope,
    constructor: Scope,
    macro_name: Scope,
    entity_macro: Scope,
    preprocessor: Scope,
    support_constant: Scope,
    builtin_constant: Scope,
    support_function: Scope,
    builtin_function: Scope,
    support_type: Scope,
    support_type_builtin: Scope,
    support_class_builtin: Scope,
    support_namespace: Scope,
    variable_function: Scope,
    variable: Scope,
    variable_parameter: Scope,
    variable_property: Scope,
    variable_member: Scope,
    variable_language: Scope,
    builtin_variable: Scope,
    constant: Scope,
    constant_character: Scope,
    punctuation: Scope,
    punctuation_definition: Scope,
    punctuation_separator: Scope,
    punctuation_terminator: Scope,
    punctuation_section: Scope,
    punctuation_accessor: Scope,
    entity_field: Scope,
}

impl TokenScopes {
    fn new() -> Self {
        Self {
            comment: scope("comment"),
            string: scope("string"),
            string_escape: scope("constant.character.escape"),
            string_regexp: scope("string.regexp"),
            number: scope("constant.numeric"),
            bool: scope("constant.language.boolean"),
            null: scope("constant.language.null"),
            keyword: scope("keyword"),
            keyword_control: scope("keyword.control"),
            keyword_operator: scope("keyword.operator"),
            storage_modifier: scope("storage.modifier"),
            storage_type: scope("storage.type"),
            type_builtin: scope("storage.type.primitive"),
            entity_tag: scope("entity.name.tag"),
            entity_type: scope("entity.name.type"),
            entity_class: scope("entity.name.class"),
            entity_struct: scope("entity.name.struct"),
            entity_enum: scope("entity.name.enum"),
            entity_union: scope("entity.name.union"),
            entity_interface: scope("entity.name.interface"),
            entity_trait: scope("entity.name.trait"),
            namespace: scope("entity.name.namespace"),
            attribute_name: scope("entity.other.attribute-name"),
            decorator: scope("meta.annotation"),
            entity_function: scope("entity.name.function"),
            constructor: scope("entity.name.function.constructor"),
            macro_name: scope("entity.name.function.preprocessor"),
            entity_macro: scope("entity.name.macro"),
            preprocessor: scope("meta.preprocessor"),
            support_constant: scope("support.constant"),
            builtin_constant: scope("support.constant.builtin"),
            support_function: scope("support.function"),
            builtin_function: scope("support.function.builtin"),
            support_type: scope("support.type"),
            support_type_builtin: scope("support.type.builtin"),
            support_class_builtin: scope("support.class.builtin"),
            support_namespace: scope("support.namespace"),
            variable_function: scope("variable.function"),
            variable: scope("variable"),
            variable_parameter: scope("variable.parameter"),
            variable_property: scope("variable.other.property"),
            variable_member: scope("variable.other.member"),
            variable_language: scope("variable.language"),
            builtin_variable: scope("support.variable"),
            constant: scope("constant"),
            constant_character: scope("constant.character"),
            punctuation: scope("punctuation"),
            punctuation_definition: scope("punctuation.definition"),
            punctuation_separator: scope("punctuation.separator"),
            punctuation_terminator: scope("punctuation.terminator"),
            punctuation_section: scope("punctuation.section"),
            punctuation_accessor: scope("punctuation.accessor"),
            entity_field: scope("entity.name.field"),
        }
    }
}

fn scope(value: &str) -> Scope {
    Scope::new(value).expect("invalid syntect scope selector")
}

fn escape_lines(lines: Vec<&str>) -> Vec<String> {
    lines.into_iter().map(escape_html).collect()
}

fn escape_html(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    push_escaped(&mut escaped, text);
    escaped
}

fn push_escaped(out: &mut String, text: &str) {
    for ch in text.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
}
