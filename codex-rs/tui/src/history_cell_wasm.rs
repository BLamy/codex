use std::path::Path;
use std::path::PathBuf;

use crate::render::line_utils::prefix_lines;
use crate::render::line_utils::push_owned_lines;
use crate::style::user_message_style;
use crate::terminal_hyperlinks::HyperlinkLine;
use crate::terminal_hyperlinks::prefix_hyperlink_lines;
use crate::terminal_hyperlinks::visible_lines;
use crate::ui_consts::LIVE_PREFIX_COLS;
use crate::wrapping::RtOptions;
use crate::wrapping::adaptive_wrap_line;
use crate::wrapping::adaptive_wrap_lines;
use codex_protocol::plan_tool::PlanItemArg;
use codex_protocol::plan_tool::StepStatus;
use codex_protocol::plan_tool::UpdatePlanArgs;
use codex_protocol::user_input::TextElement;
use ratatui::style::Style;
use ratatui::style::Styled;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::text::Span;
use ratatui::text::Text;
use ratatui::widgets::Paragraph;
use ratatui::widgets::Wrap;
use unicode_width::UnicodeWidthStr;

pub(crate) const SESSION_HEADER_MAX_INNER_WIDTH: usize = 56;

pub(crate) fn card_inner_width(width: u16, max_inner_width: usize) -> Option<usize> {
    if width < 4 {
        return None;
    }
    Some(std::cmp::min(
        width.saturating_sub(4) as usize,
        max_inner_width,
    ))
}

pub(crate) fn with_border(lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    with_border_internal(lines, None)
}

fn with_border_internal(
    lines: Vec<Line<'static>>,
    forced_inner_width: Option<usize>,
) -> Vec<Line<'static>> {
    let max_line_width = lines
        .iter()
        .map(|line| {
            line.iter()
                .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
                .sum::<usize>()
        })
        .max()
        .unwrap_or(0);
    let content_width = forced_inner_width
        .unwrap_or(max_line_width)
        .max(max_line_width);

    let mut out = Vec::with_capacity(lines.len() + 2);
    let border_inner_width = content_width + 2;
    out.push(vec![format!("╭{}╮", "─".repeat(border_inner_width)).dim()].into());

    for line in lines.into_iter() {
        let used_width: usize = line
            .iter()
            .map(|span| UnicodeWidthStr::width(span.content.as_ref()))
            .sum();
        let mut spans: Vec<Span<'static>> = Vec::with_capacity(line.spans.len() + 4);
        spans.push(Span::from("│ ").dim());
        spans.extend(line);
        if used_width < content_width {
            spans.push(Span::from(" ".repeat(content_width - used_width)).dim());
        }
        spans.push(Span::from(" │").dim());
        out.push(Line::from(spans));
    }

    out.push(vec![format!("╰{}╯", "─".repeat(border_inner_width)).dim()].into());
    out
}

pub(crate) fn raw_lines_from_source(source: &str) -> Vec<Line<'static>> {
    if source.is_empty() {
        return Vec::new();
    }

    let mut parts = source.split('\n').collect::<Vec<_>>();
    if source.ends_with('\n') {
        parts.pop();
    }

    parts
        .into_iter()
        .map(|line| Line::from(line.to_string()))
        .collect()
}

pub(crate) trait HistoryCell: std::fmt::Debug + Send + Sync {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>>;

    fn raw_lines(&self) -> Vec<Line<'static>>;

    fn transcript_lines(&self, width: u16) -> Vec<Line<'static>> {
        self.display_lines(width)
    }

    fn desired_height(&self, width: u16) -> u16 {
        Paragraph::new(Text::from(self.display_lines(width)))
            .wrap(Wrap { trim: false })
            .line_count(width)
            .try_into()
            .unwrap_or(0)
    }
}

#[derive(Debug)]
pub(crate) struct PlainHistoryCell {
    lines: Vec<Line<'static>>,
}

impl PlainHistoryCell {
    pub(crate) fn new(lines: Vec<Line<'static>>) -> Self {
        Self { lines }
    }
}

impl HistoryCell for PlainHistoryCell {
    fn display_lines(&self, _width: u16) -> Vec<Line<'static>> {
        self.lines.clone()
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        self.lines.clone()
    }
}

#[derive(Debug)]
pub(crate) struct UserHistoryCell {
    pub message: String,
    pub text_elements: Vec<TextElement>,
    #[allow(dead_code)]
    pub local_image_paths: Vec<PathBuf>,
    pub remote_image_urls: Vec<String>,
}

fn trim_trailing_blank_lines(mut lines: Vec<Line<'static>>) -> Vec<Line<'static>> {
    while lines
        .last()
        .is_some_and(|line| line.spans.iter().all(|span| span.content.trim().is_empty()))
    {
        lines.pop();
    }
    lines
}

impl HistoryCell for UserHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let wrap_width = width.saturating_sub(LIVE_PREFIX_COLS + 1).max(1);
        let style = user_message_style();
        let message_without_trailing_newlines = self.message.trim_end_matches(['\r', '\n']);
        let wrapped = adaptive_wrap_lines(
            message_without_trailing_newlines
                .split('\n')
                .map(|line| Line::from(line.to_string()).style(style)),
            RtOptions::new(usize::from(wrap_width))
                .wrap_algorithm(textwrap::WrapAlgorithm::FirstFit),
        );
        let wrapped = trim_trailing_blank_lines(wrapped);
        if wrapped.is_empty() {
            return Vec::new();
        }

        let mut lines: Vec<Line<'static>> = vec![Line::from("").style(style)];
        lines.extend(prefix_lines(wrapped, "› ".bold().dim(), Span::from("  ")));
        lines.push(Line::from("").style(style));
        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        raw_lines_from_source(self.message.trim_end_matches(['\r', '\n']))
    }
}

#[derive(Debug)]
pub(crate) struct AgentMarkdownCell {
    markdown_source: String,
    cwd: PathBuf,
}

impl AgentMarkdownCell {
    pub(crate) fn new(markdown_source: String, cwd: &Path) -> Self {
        Self {
            markdown_source,
            cwd: cwd.to_path_buf(),
        }
    }

    fn display_hyperlink_lines(&self, width: u16) -> Vec<HyperlinkLine> {
        let Some(wrap_width) = crate::width::usable_content_width_u16(width, 2) else {
            return prefix_hyperlink_lines(
                vec![HyperlinkLine::new(Line::default())],
                "• ".dim(),
                "  ".into(),
            );
        };

        let lines = crate::markdown::render_markdown_agent_with_links_and_cwd(
            &self.markdown_source,
            Some(wrap_width),
            Some(self.cwd.as_path()),
        );
        prefix_hyperlink_lines(lines, "• ".dim(), "  ".into())
    }
}

impl HistoryCell for AgentMarkdownCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        visible_lines(self.display_hyperlink_lines(width))
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        raw_lines_from_source(&self.markdown_source)
    }
}

#[derive(Debug)]
pub(crate) struct SessionHeaderHistoryCell {
    version: Option<String>,
    model: String,
    model_style: Style,
    directory: PathBuf,
}

impl SessionHeaderHistoryCell {
    pub(crate) fn new(
        model: String,
        directory: impl Into<PathBuf>,
        version: Option<String>,
    ) -> Self {
        Self {
            version,
            model,
            model_style: Style::default(),
            directory: directory.into(),
        }
    }

    fn format_directory(&self, max_width: Option<usize>) -> String {
        Self::format_directory_inner(&self.directory, max_width)
    }

    pub(crate) fn format_directory_inner(directory: &Path, max_width: Option<usize>) -> String {
        let formatted = directory.display().to_string();

        if let Some(max_width) = max_width {
            if max_width == 0 {
                return String::new();
            }
            if UnicodeWidthStr::width(formatted.as_str()) > max_width {
                return crate::text_formatting::center_truncate_path(&formatted, max_width);
            }
        }

        formatted
    }
}

impl HistoryCell for SessionHeaderHistoryCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let Some(inner_width) = card_inner_width(width, SESSION_HEADER_MAX_INNER_WIDTH) else {
            return Vec::new();
        };

        let make_row = |spans: Vec<Span<'static>>| Line::from(spans);

        let mut title_spans: Vec<Span<'static>> =
            vec![Span::from(">_ ").dim(), Span::from("OpenAI Codex").bold()];
        if let Some(version) = self.version.as_deref() {
            title_spans.push(Span::from(" ").dim());
            title_spans.push(Span::from(format!("(v{version})")).dim());
        }

        const CHANGE_MODEL_HINT_COMMAND: &str = "/model";
        const CHANGE_MODEL_HINT_EXPLANATION: &str = " to change";
        const DIR_LABEL: &str = "directory:";
        let label_width = DIR_LABEL.len();

        let model_label = format!(
            "{model_label:<label_width$}",
            model_label = "model:",
            label_width = label_width
        );
        let model_spans: Vec<Span<'static>> = vec![
            Span::from(format!("{model_label} ")).dim(),
            Span::styled(self.model.clone(), self.model_style),
            "   ".dim(),
            CHANGE_MODEL_HINT_COMMAND.cyan(),
            CHANGE_MODEL_HINT_EXPLANATION.dim(),
        ];

        let dir_label = format!("{DIR_LABEL:<label_width$}");
        let dir_prefix = format!("{dir_label} ");
        let dir_prefix_width = UnicodeWidthStr::width(dir_prefix.as_str());
        let dir_max_width = inner_width.saturating_sub(dir_prefix_width);
        let dir = self.format_directory(Some(dir_max_width));
        let dir_spans = vec![Span::from(dir_prefix).dim(), Span::from(dir)];

        with_border(vec![
            make_row(title_spans),
            make_row(Vec::new()),
            make_row(model_spans),
            make_row(dir_spans),
        ])
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let title = match self.version.as_deref() {
            Some(version) => format!("OpenAI Codex (v{version})"),
            None => "OpenAI Codex".to_string(),
        };
        vec![
            Line::from(title),
            Line::from(format!("model: {}", self.model)),
            Line::from(format!(
                "directory: {}",
                self.format_directory(/*max_width*/ None)
            )),
        ]
    }
}

pub(crate) fn new_plan_update(update: UpdatePlanArgs) -> PlanUpdateCell {
    let UpdatePlanArgs { explanation, plan } = update;
    PlanUpdateCell { explanation, plan }
}

pub(crate) fn new_agent_message(markdown_source: String, cwd: &Path) -> AgentMarkdownCell {
    AgentMarkdownCell::new(markdown_source, cwd)
}

#[derive(Debug)]
pub(crate) struct PlanUpdateCell {
    explanation: Option<String>,
    plan: Vec<PlanItemArg>,
}

impl HistoryCell for PlanUpdateCell {
    fn display_lines(&self, width: u16) -> Vec<Line<'static>> {
        let render_note = |text: &str| -> Vec<Line<'static>> {
            let wrap_width = width.saturating_sub(4).max(1) as usize;
            let note = Line::from(text.to_string().dim().italic());
            let wrapped = adaptive_wrap_line(&note, RtOptions::new(wrap_width));
            let mut out = Vec::new();
            push_owned_lines(&wrapped, &mut out);
            out
        };

        let render_step = |status: &StepStatus, text: &str| -> Vec<Line<'static>> {
            let (box_str, step_style) = match status {
                StepStatus::Completed => ("✔ ", Style::default().crossed_out().dim()),
                StepStatus::InProgress => ("□ ", Style::default().cyan().bold()),
                StepStatus::Pending => ("□ ", Style::default().dim()),
            };

            let opts = RtOptions::new(width.saturating_sub(4).max(1) as usize)
                .initial_indent(box_str.into())
                .subsequent_indent("  ".into());
            let step = Line::from(text.to_string().set_style(step_style));
            let wrapped = adaptive_wrap_line(&step, opts);
            let mut out = Vec::new();
            push_owned_lines(&wrapped, &mut out);
            out
        };

        let mut lines: Vec<Line<'static>> = vec![vec!["• ".dim(), "Updated Plan".bold()].into()];

        let mut indented_lines = vec![];
        let note = self
            .explanation
            .as_ref()
            .map(|s| s.trim())
            .filter(|t| !t.is_empty());
        if let Some(expl) = note {
            indented_lines.extend(render_note(expl));
        }

        if self.plan.is_empty() {
            indented_lines.push(Line::from("(no steps provided)".dim().italic()));
        } else {
            for PlanItemArg { step, status } in self.plan.iter() {
                indented_lines.extend(render_step(status, step));
            }
        }
        lines.extend(prefix_lines(indented_lines, "  └ ".dim(), "    ".into()));

        lines
    }

    fn raw_lines(&self) -> Vec<Line<'static>> {
        let mut lines = vec![Line::from("Updated Plan")];
        if let Some(explanation) = self
            .explanation
            .as_ref()
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            lines.extend(raw_lines_from_source(explanation));
        }
        if self.plan.is_empty() {
            lines.push(Line::from("(no steps provided)"));
        } else {
            for PlanItemArg { step, status } in &self.plan {
                lines.push(Line::from(format!("{status:?}: {step}")));
            }
        }
        lines
    }
}

pub(crate) fn new_info_event(message: String, hint: Option<String>) -> PlainHistoryCell {
    let mut lines = raw_lines_from_source(&message);
    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    let mut out = Vec::with_capacity(lines.len());
    for (index, line) in lines.into_iter().enumerate() {
        let mut spans = Vec::with_capacity(line.spans.len() + 3);
        spans.push(if index == 0 {
            "• ".dim()
        } else {
            Span::from("  ")
        });
        spans.extend(line.spans);
        if index == 0
            && let Some(hint) = hint.as_deref()
        {
            spans.push(" ".into());
            spans.push(hint.to_string().dark_gray());
        }
        out.push(Line::from(spans).style(line.style));
    }

    PlainHistoryCell::new(out)
}

pub(crate) fn new_error_event(message: String) -> PlainHistoryCell {
    let mut lines = raw_lines_from_source(&message);
    if lines.is_empty() {
        lines.push(Line::from(""));
    }

    let out = lines
        .into_iter()
        .enumerate()
        .map(|(index, line)| {
            let mut spans = Vec::with_capacity(line.spans.len() + 1);
            spans.push(if index == 0 {
                "■ ".red()
            } else {
                Span::from("  ")
            });
            spans.extend(line.spans.into_iter().map(|span| span.red()));
            Line::from(spans).style(line.style)
        })
        .collect();

    PlainHistoryCell::new(out)
}
