use codex_protocol::ThreadId;
use codex_protocol::items::HookPromptFragment;
use codex_utils_output_truncation::TruncationPolicy;
use codex_utils_output_truncation::approx_token_count;
use codex_utils_output_truncation::formatted_truncate_text;

pub(crate) const DEFAULT_HOOK_OUTPUT_TOKEN_LIMIT: usize = 2_500;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct AdditionalContextLimit {
    token_limit: usize,
}

impl AdditionalContextLimit {
    pub(crate) fn from_config(value: Option<usize>) -> Self {
        Self {
            token_limit: value.unwrap_or(DEFAULT_HOOK_OUTPUT_TOKEN_LIMIT),
        }
    }
}

impl Default for AdditionalContextLimit {
    fn default() -> Self {
        Self::from_config(/*value*/ None)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct AdditionalContext {
    pub text: String,
    pub limit: AdditionalContextLimit,
}

#[derive(Clone, Default)]
pub(crate) struct HookOutputSpiller;

impl HookOutputSpiller {
    pub(crate) fn new() -> Self {
        Self
    }

    pub(crate) async fn maybe_spill_text(&self, _thread_id: ThreadId, text: String) -> String {
        Self::truncate_to_limit(text, AdditionalContextLimit::default())
    }

    fn truncate_to_limit(text: String, limit: AdditionalContextLimit) -> String {
        if limit.token_limit == 0 || approx_token_count(&text) <= limit.token_limit {
            return text;
        }
        formatted_truncate_text(&text, TruncationPolicy::Tokens(limit.token_limit))
    }

    pub(crate) async fn maybe_spill_additional_contexts(
        &self,
        _thread_id: ThreadId,
        contexts: Vec<AdditionalContext>,
    ) -> Vec<String> {
        contexts
            .into_iter()
            .map(|context| Self::truncate_to_limit(context.text, context.limit))
            .collect()
    }

    pub(crate) async fn maybe_spill_prompt_fragments(
        &self,
        thread_id: ThreadId,
        fragments: Vec<HookPromptFragment>,
    ) -> Vec<HookPromptFragment> {
        let mut truncated = Vec::with_capacity(fragments.len());
        for fragment in fragments {
            truncated.push(HookPromptFragment {
                text: self.maybe_spill_text(thread_id, fragment.text).await,
                hook_run_id: fragment.hook_run_id,
            });
        }
        truncated
    }
}
