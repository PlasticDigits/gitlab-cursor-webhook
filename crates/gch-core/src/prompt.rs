// SPDX-License-Identifier: AGPL-3.0-or-later

use std::collections::HashMap;

/// Variables available for prompt template substitution.
#[derive(Debug, Clone, Default)]
pub struct PromptContext {
    pub vars: HashMap<String, String>,
}

impl PromptContext {
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.vars.insert(key.into(), value.into());
    }
}

/// Render a prompt template replacing `{{key}}` placeholders.
pub fn render_prompt(template: &str, ctx: &PromptContext) -> String {
    let mut out = template.to_string();
    for (key, value) in &ctx.vars {
        let placeholder = format!("{{{{{key}}}}}");
        out = out.replace(&placeholder, value);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_placeholders() {
        let mut ctx = PromptContext::default();
        ctx.insert("title", "Fix bug");
        ctx.insert("iid", "42");
        let rendered = render_prompt("Review MR #{{iid}}: {{title}}", &ctx);
        assert_eq!(rendered, "Review MR #42: Fix bug");
    }
}
