use super::message_tool::FollowupTaskArgs;
use super::message_tool::MessageDeliveryMode;
use super::message_tool::handle_message_string_tool;
use super::*;
use crate::tools::handlers::multi_agents_spec::create_followup_task_tool;
use codex_tools::ToolSpec;

pub(crate) struct Handler;

#[cfg_attr(not(target_arch = "wasm32"), async_trait::async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait::async_trait(?Send))]
impl ToolExecutor<ToolInvocation> for Handler {
    fn tool_name(&self) -> ToolName {
        ToolName::plain("followup_task")
    }

    fn spec(&self) -> ToolSpec {
        create_followup_task_tool()
    }

    async fn handle(
        &self,
        invocation: ToolInvocation,
    ) -> Result<Box<dyn crate::tools::context::ToolOutput>, FunctionCallError> {
        let arguments = function_arguments(invocation.payload.clone())?;
        let args: FollowupTaskArgs = parse_arguments(&arguments)?;
        handle_message_string_tool(
            invocation,
            MessageDeliveryMode::TriggerTurn,
            args.target,
            args.message,
        )
        .await
        .map(boxed_tool_output)
    }
}

impl CoreToolRuntime for Handler {
    fn matches_kind(&self, payload: &ToolPayload) -> bool {
        matches!(payload, ToolPayload::Function { .. })
    }
}
