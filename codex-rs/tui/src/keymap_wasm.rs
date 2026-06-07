use crate::key_hint;
use crate::key_hint::KeyBinding;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;

#[derive(Clone, Debug)]
pub(crate) struct RuntimeKeymap {
    pub(crate) app: AppKeymap,
    pub(crate) chat: ChatKeymap,
    pub(crate) composer: ComposerKeymap,
    pub(crate) editor: EditorKeymap,
    pub(crate) vim_normal: VimNormalKeymap,
    pub(crate) vim_operator: VimOperatorKeymap,
    pub(crate) vim_text_object: VimTextObjectKeymap,
    pub(crate) list: ListKeymap,
}

#[derive(Clone, Debug)]
pub(crate) struct AppKeymap {
    pub(crate) open_transcript: Vec<KeyBinding>,
    pub(crate) open_external_editor: Vec<KeyBinding>,
}

#[derive(Clone, Debug)]
pub(crate) struct ChatKeymap {
    pub(crate) decrease_reasoning_effort: Vec<KeyBinding>,
    pub(crate) increase_reasoning_effort: Vec<KeyBinding>,
}

#[derive(Clone, Debug)]
pub(crate) struct ComposerKeymap {
    pub(crate) submit: Vec<KeyBinding>,
    pub(crate) queue: Vec<KeyBinding>,
    pub(crate) toggle_shortcuts: Vec<KeyBinding>,
    pub(crate) history_search_previous: Vec<KeyBinding>,
    pub(crate) history_search_next: Vec<KeyBinding>,
}

#[derive(Clone, Debug)]
pub(crate) struct EditorKeymap {
    pub(crate) insert_newline: Vec<KeyBinding>,
    pub(crate) move_left: Vec<KeyBinding>,
    pub(crate) move_right: Vec<KeyBinding>,
    pub(crate) move_up: Vec<KeyBinding>,
    pub(crate) move_down: Vec<KeyBinding>,
    pub(crate) move_word_left: Vec<KeyBinding>,
    pub(crate) move_word_right: Vec<KeyBinding>,
    pub(crate) move_line_start: Vec<KeyBinding>,
    pub(crate) move_line_end: Vec<KeyBinding>,
    pub(crate) delete_backward: Vec<KeyBinding>,
    pub(crate) delete_forward: Vec<KeyBinding>,
    pub(crate) delete_backward_word: Vec<KeyBinding>,
    pub(crate) delete_forward_word: Vec<KeyBinding>,
    pub(crate) kill_line_start: Vec<KeyBinding>,
    pub(crate) kill_whole_line: Vec<KeyBinding>,
    pub(crate) kill_line_end: Vec<KeyBinding>,
    pub(crate) yank: Vec<KeyBinding>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VimNormalKeymap {
    pub(crate) enter_insert: Vec<KeyBinding>,
    pub(crate) append_after_cursor: Vec<KeyBinding>,
    pub(crate) append_line_end: Vec<KeyBinding>,
    pub(crate) insert_line_start: Vec<KeyBinding>,
    pub(crate) open_line_below: Vec<KeyBinding>,
    pub(crate) open_line_above: Vec<KeyBinding>,
    pub(crate) move_left: Vec<KeyBinding>,
    pub(crate) move_right: Vec<KeyBinding>,
    pub(crate) move_up: Vec<KeyBinding>,
    pub(crate) move_down: Vec<KeyBinding>,
    pub(crate) move_word_forward: Vec<KeyBinding>,
    pub(crate) move_word_backward: Vec<KeyBinding>,
    pub(crate) move_word_end: Vec<KeyBinding>,
    pub(crate) move_line_start: Vec<KeyBinding>,
    pub(crate) move_line_end: Vec<KeyBinding>,
    pub(crate) delete_char: Vec<KeyBinding>,
    pub(crate) substitute_char: Vec<KeyBinding>,
    pub(crate) delete_to_line_end: Vec<KeyBinding>,
    pub(crate) change_to_line_end: Vec<KeyBinding>,
    pub(crate) yank_line: Vec<KeyBinding>,
    pub(crate) paste_after: Vec<KeyBinding>,
    pub(crate) start_delete_operator: Vec<KeyBinding>,
    pub(crate) start_yank_operator: Vec<KeyBinding>,
    pub(crate) start_change_operator: Vec<KeyBinding>,
    pub(crate) cancel_operator: Vec<KeyBinding>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VimOperatorKeymap {
    pub(crate) delete_line: Vec<KeyBinding>,
    pub(crate) yank_line: Vec<KeyBinding>,
    pub(crate) motion_left: Vec<KeyBinding>,
    pub(crate) motion_right: Vec<KeyBinding>,
    pub(crate) motion_up: Vec<KeyBinding>,
    pub(crate) motion_down: Vec<KeyBinding>,
    pub(crate) motion_word_forward: Vec<KeyBinding>,
    pub(crate) motion_word_backward: Vec<KeyBinding>,
    pub(crate) motion_word_end: Vec<KeyBinding>,
    pub(crate) motion_line_start: Vec<KeyBinding>,
    pub(crate) motion_line_end: Vec<KeyBinding>,
    pub(crate) select_inner_text_object: Vec<KeyBinding>,
    pub(crate) select_around_text_object: Vec<KeyBinding>,
    pub(crate) cancel: Vec<KeyBinding>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct VimTextObjectKeymap {
    pub(crate) word: Vec<KeyBinding>,
    pub(crate) big_word: Vec<KeyBinding>,
    pub(crate) parentheses: Vec<KeyBinding>,
    pub(crate) brackets: Vec<KeyBinding>,
    pub(crate) braces: Vec<KeyBinding>,
    pub(crate) double_quote: Vec<KeyBinding>,
    pub(crate) single_quote: Vec<KeyBinding>,
    pub(crate) backtick: Vec<KeyBinding>,
    pub(crate) cancel: Vec<KeyBinding>,
}

#[derive(Clone, Debug)]
pub(crate) struct ListKeymap {
    pub(crate) accept: Vec<KeyBinding>,
    pub(crate) cancel: Vec<KeyBinding>,
}

impl RuntimeKeymap {
    pub(crate) fn defaults() -> Self {
        Self {
            app: AppKeymap {
                open_transcript: vec![key_hint::ctrl(KeyCode::Char('t'))],
                open_external_editor: vec![key_hint::ctrl(KeyCode::Char('g'))],
            },
            chat: ChatKeymap {
                decrease_reasoning_effort: vec![
                    key_hint::alt(KeyCode::Char(',')),
                    key_hint::shift(KeyCode::Down),
                ],
                increase_reasoning_effort: vec![
                    key_hint::alt(KeyCode::Char('.')),
                    key_hint::shift(KeyCode::Up),
                ],
            },
            composer: ComposerKeymap {
                submit: vec![key_hint::plain(KeyCode::Enter)],
                queue: vec![key_hint::plain(KeyCode::Tab)],
                toggle_shortcuts: vec![
                    key_hint::plain(KeyCode::Char('?')),
                    key_hint::shift(KeyCode::Char('?')),
                ],
                history_search_previous: vec![key_hint::ctrl(KeyCode::Char('r'))],
                history_search_next: vec![key_hint::ctrl(KeyCode::Char('s'))],
            },
            editor: EditorKeymap::defaults(),
            vim_normal: VimNormalKeymap::default(),
            vim_operator: VimOperatorKeymap::default(),
            vim_text_object: VimTextObjectKeymap::default(),
            list: ListKeymap {
                accept: vec![key_hint::plain(KeyCode::Enter)],
                cancel: vec![key_hint::plain(KeyCode::Esc)],
            },
        }
    }
}

impl EditorKeymap {
    fn defaults() -> Self {
        Self {
            insert_newline: vec![
                key_hint::ctrl(KeyCode::Char('j')),
                key_hint::ctrl(KeyCode::Char('m')),
                key_hint::plain(KeyCode::Enter),
                key_hint::shift(KeyCode::Enter),
                key_hint::alt(KeyCode::Enter),
            ],
            move_left: vec![
                key_hint::plain(KeyCode::Left),
                key_hint::ctrl(KeyCode::Char('b')),
            ],
            move_right: vec![
                key_hint::plain(KeyCode::Right),
                key_hint::ctrl(KeyCode::Char('f')),
            ],
            move_up: vec![
                key_hint::plain(KeyCode::Up),
                key_hint::ctrl(KeyCode::Char('p')),
            ],
            move_down: vec![
                key_hint::plain(KeyCode::Down),
                key_hint::ctrl(KeyCode::Char('n')),
            ],
            move_word_left: vec![
                key_hint::alt(KeyCode::Char('b')),
                KeyBinding::new(KeyCode::Left, KeyModifiers::ALT),
                KeyBinding::new(KeyCode::Left, KeyModifiers::CONTROL),
            ],
            move_word_right: vec![
                key_hint::alt(KeyCode::Char('f')),
                KeyBinding::new(KeyCode::Right, KeyModifiers::ALT),
                KeyBinding::new(KeyCode::Right, KeyModifiers::CONTROL),
            ],
            move_line_start: vec![
                key_hint::plain(KeyCode::Home),
                key_hint::ctrl(KeyCode::Char('a')),
            ],
            move_line_end: vec![
                key_hint::plain(KeyCode::End),
                key_hint::ctrl(KeyCode::Char('e')),
            ],
            delete_backward: vec![
                key_hint::plain(KeyCode::Backspace),
                key_hint::shift(KeyCode::Backspace),
                key_hint::ctrl(KeyCode::Char('h')),
            ],
            delete_forward: vec![
                key_hint::plain(KeyCode::Delete),
                key_hint::shift(KeyCode::Delete),
                key_hint::ctrl(KeyCode::Char('d')),
            ],
            delete_backward_word: vec![
                key_hint::alt(KeyCode::Backspace),
                key_hint::ctrl(KeyCode::Backspace),
                KeyBinding::new(
                    KeyCode::Backspace,
                    KeyModifiers::CONTROL | KeyModifiers::SHIFT,
                ),
                key_hint::ctrl(KeyCode::Char('w')),
                KeyBinding::new(
                    KeyCode::Char('h'),
                    KeyModifiers::CONTROL | KeyModifiers::ALT,
                ),
            ],
            delete_forward_word: vec![
                key_hint::alt(KeyCode::Delete),
                key_hint::ctrl(KeyCode::Delete),
                KeyBinding::new(KeyCode::Delete, KeyModifiers::CONTROL | KeyModifiers::SHIFT),
                key_hint::alt(KeyCode::Char('d')),
            ],
            kill_line_start: vec![key_hint::ctrl(KeyCode::Char('u'))],
            kill_whole_line: Vec::new(),
            kill_line_end: vec![key_hint::ctrl(KeyCode::Char('k'))],
            yank: vec![key_hint::ctrl(KeyCode::Char('y'))],
        }
    }
}

impl Default for RuntimeKeymap {
    fn default() -> Self {
        Self::defaults()
    }
}

pub(crate) fn primary_binding(bindings: &[KeyBinding]) -> Option<KeyBinding> {
    bindings.first().copied()
}
