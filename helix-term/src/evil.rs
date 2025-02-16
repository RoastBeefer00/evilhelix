use std::ffi::OsString;
use std::num::NonZeroUsize;
use std::path::{Path, PathBuf};

use arc_swap::ArcSwap;
use ropey::Rope;
use std::sync::Arc;

use helix_core::match_brackets::find_matching_bracket;
use helix_core::movement::{self, Movement};
use helix_core::{textobject, Range};
use helix_stdx::env::current_working_dir;
use helix_view::document::{Document, Mode};
use helix_view::editor::{Action, Config};
use helix_view::info::Info;
use helix_view::Editor;

use crate::commands::{
    change_selection, delete_selection, enter_netrw_mode, extend_to_line_bounds, extend_word_impl,
    find_char_impl_forward, find_next_char_impl, goto_line_end_impl, select_line_below,
    select_mode, yank, Context, Operation,
};

pub(crate) fn change_to_end_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_end);
    change_selection(cx);
}

pub(crate) fn change_to_end_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_end);
    change_selection(cx);
}

pub(crate) fn change_to_beginning_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_start);
    change_selection(cx);
}

pub(crate) fn change_to_beginning_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_long_word_start);
    change_selection(cx);
}

pub(crate) fn delete_to_end_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_end);
    delete_selection(cx);
}

pub(crate) fn delete_to_end_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_end);
    delete_selection(cx);
}

pub(crate) fn delete_to_beginning_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_start);
    delete_selection(cx);
}

pub(crate) fn delete_to_beginning_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_long_word_start);
    delete_selection(cx);
}

pub(crate) fn select_to_start_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_start);
    select_mode(cx);
}

pub(crate) fn select_to_start_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_start);
    select_mode(cx);
}

pub(crate) fn select_to_end_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_end);
    select_mode(cx);
}

pub(crate) fn select_to_end_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_end);
    select_mode(cx);
}

pub(crate) fn select_to_beginning_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_start);
    select_mode(cx);
}

pub(crate) fn select_to_beginning_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_long_word_start);
    select_mode(cx);
}

pub(crate) fn yank_to_end_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_word_end);
    yank(cx);
}

pub(crate) fn yank_to_end_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_next_long_word_end);
    yank(cx);
}

pub(crate) fn yank_to_beginning_of_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_word_start);
    yank(cx);
}

pub(crate) fn yank_to_beginning_of_long_word(cx: &mut Context) {
    extend_word_impl(cx, movement::move_prev_long_word_start);
    yank(cx);
}

pub(crate) fn change_line(cx: &mut Context) {
    let count = cx.count();
    extend_to_line_bounds(cx);
    if cx.count() > 1 {
        cx.count = NonZeroUsize::new(1);
        for _ in 0..count - 1 {
            select_line_below(cx);
            extend_to_line_bounds(cx);
        }
    }
    change_selection(cx);
}

pub(crate) fn delete_line(cx: &mut Context) {
    let count = cx.count();
    for _ in 0..count {
        extend_to_line_bounds(cx);
        delete_selection(cx);
    }
}

pub(crate) fn yank_line(cx: &mut Context) {
    let count = cx.count();
    extend_to_line_bounds(cx);
    if cx.count() > 1 {
        cx.count = NonZeroUsize::new(1);
        for _ in 0..count - 1 {
            select_line_below(cx);
            extend_to_line_bounds(cx);
        }
    }
    yank(cx);
}

pub(crate) fn change_to_end_of_line(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    goto_line_end_impl(view, doc, Movement::Extend);
    change_selection(cx);
}

pub(crate) fn delete_to_end_of_line(cx: &mut Context) {
    let (view, doc) = current!(cx.editor);
    goto_line_end_impl(view, doc, Movement::Extend);
    delete_selection(cx);
}

pub(crate) fn goto_matching_pair(cx: &mut Context) {
    let mode = cx.editor.mode();
    let (view, doc) = current!(cx.editor);
    if let Some(syntax) = doc.syntax() {
        let text = doc.text().slice(..);
        let original_pos = doc.selection(view.id).primary().cursor(text);
        if let Some(pos) = find_matching_bracket(syntax, text, original_pos) {
            let selection = doc.selection(view.id).clone().transform(|mut range| {
                if mode == Mode::Select {
                    if pos > original_pos {
                        range.anchor = original_pos;
                        range.head = pos + 1;
                    } else {
                        range.anchor = original_pos + 1;
                        range.head = pos;
                    }
                } else {
                    range.anchor = pos;
                    range.head = pos + 1;
                }
                range
            });
            doc.set_selection(view.id, selection);
        }
    }
}

pub(crate) fn select_textobject_around(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Around, Operation::Select);
}

pub(crate) fn select_textobject_inner(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Inside, Operation::Select);
}

pub(crate) fn change_textobject_around(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Around, Operation::Change);
}

pub(crate) fn change_textobject_inner(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Inside, Operation::Change);
}

pub(crate) fn delete_textobject_around(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Around, Operation::Delete);
}

pub(crate) fn delete_textobject_inner(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Inside, Operation::Delete);
}

pub(crate) fn yank_textobject_around(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Around, Operation::Yank);
}

pub(crate) fn yank_textobject_inner(cx: &mut Context) {
    textobject_impl(cx, textobject::TextObject::Inside, Operation::Yank);
}

fn textobject_impl(cx: &mut Context, objtype: textobject::TextObject, op: Operation) {
    let count = cx.count();

    cx.on_next_key(move |cx, event| {
        cx.editor.autoinfo = None;
        if let Some(ch) = event.char() {
            let textobject = move |editor: &mut Editor| {
                let (view, doc) = current!(editor);
                let text = doc.text().slice(..);

                let textobject_treesitter = |obj_name: &str, range: Range| -> Range {
                    let (lang_config, syntax) = match doc.language_config().zip(doc.syntax()) {
                        Some(t) => t,
                        None => return range,
                    };
                    textobject::textobject_treesitter(
                        text,
                        range,
                        objtype,
                        obj_name,
                        syntax.tree().root_node(),
                        lang_config,
                        count,
                    )
                };

                if ch == 'g' && doc.diff_handle().is_none() {
                    editor.set_status("Diff is not available in current buffer");
                    return;
                }

                let textobject_change = |range: Range| -> Range {
                    let diff_handle = doc.diff_handle().unwrap();
                    let diff = diff_handle.load();
                    let line = range.cursor_line(text);
                    let hunk_idx = if let Some(hunk_idx) = diff.hunk_at(line as u32, false) {
                        hunk_idx
                    } else {
                        return range;
                    };
                    let hunk = diff.nth_hunk(hunk_idx).after;

                    let start = text.line_to_char(hunk.start as usize);
                    let end = text.line_to_char(hunk.end as usize);
                    Range::new(start, end).with_direction(range.direction())
                };

                let selection = doc.selection(view.id).clone().transform(|range| {
                    let r = match ch {
                        'w' => textobject::textobject_word(text, range, objtype, count, false),
                        'W' => textobject::textobject_word(text, range, objtype, count, true),
                        't' => textobject_treesitter("class", range),
                        'f' => textobject_treesitter("function", range),
                        'a' => textobject_treesitter("parameter", range),
                        'c' => textobject_treesitter("comment", range),
                        'T' => textobject_treesitter("test", range),
                        'e' => textobject_treesitter("entry", range),
                        'p' => textobject::textobject_paragraph(text, range, objtype, count),
                        'm' => textobject::textobject_pair_surround_closest(
                            doc.syntax(),
                            text,
                            range,
                            objtype,
                            count,
                        ),
                        'g' => textobject_change(range),
                        // TODO: cancel new ranges if inconsistent surround matches across lines
                        ch if !ch.is_ascii_alphanumeric() => match ch {
                            '\'' | '"' | '`' => {
                                let line = range.cursor_line(text);
                                let line_rs = text.get_line(line).unwrap();
                                textobject::textobject_pair_surround(
                                    doc.syntax(),
                                    line_rs,
                                    range,
                                    objtype,
                                    ch,
                                    count,
                                )
                            }
                            _ => textobject::textobject_pair_surround(
                                doc.syntax(),
                                text,
                                range,
                                objtype,
                                ch,
                                count,
                            ),
                        },
                        _ => range,
                    };

                    log::info!("r: {:?}", r);
                    log::info!("range: {:?}", range);
                    r
                });
                doc.set_selection(view.id, selection);
            };

            // Calculate if motion created new range
            let mut old_range = Range::new(0, 0);
            {
                let (view, doc) = current!(cx.editor);
                doc.selection(view.id).clone().transform(|range| {
                    old_range = range;
                    range
                });
            }

            cx.editor.apply_motion(textobject);

            let mut new_range = Range::new(0, 0);
            let (view, doc) = current!(cx.editor);
            doc.selection(view.id).clone().transform(|range| {
                new_range = range;
                range
            });

            log::info!("old_range: {:?}, new_range: {:?}", old_range, new_range);
            if old_range != new_range {
                log::info!("old range does not equal new range");
                match op {
                    Operation::Change => change_selection(cx),
                    Operation::Delete => delete_selection(cx),
                    Operation::Yank => yank(cx),
                    Operation::Select => select_mode(cx),
                }
            } else if !ch.is_ascii_alphanumeric() {
                let mut cont = true;
                {
                    let (view, doc) = current!(cx.editor);
                    let text = doc.text().slice(..);
                    if ch == '\'' || ch == '"' || ch == '`' {
                        doc.selection(view.id).clone().transform(|range| {
                            let line = range.cursor_line(text);
                            let line_rs = text.get_line(line).unwrap();
                            let count =
                                line_rs.chars_at(0).into_iter().filter(|c| c == &ch).count();
                            log::info!("number of {ch} on line: {count}");
                            if count < 2 {
                                cont = false;
                                return range;
                            }
                            range
                        });
                        if cont {
                            find_char_impl_forward(
                                cx.editor,
                                &find_next_char_impl,
                                true,
                                false,
                                ch,
                                1,
                            );
                        }
                    } else {
                        find_char_impl_forward(cx.editor, &find_next_char_impl, true, false, ch, 1);
                    }
                }
                if cont {
                    let (view, doc) = current!(cx.editor);
                    let text = doc.text().slice(..);
                    let selection = doc.selection(view.id).clone().transform(|mut range| {
                        if ch == '\'' || ch == '"' || ch == '`' {
                            range.anchor += 1;
                            range.head += 1;
                        }
                        textobject::textobject_pair_surround(
                            doc.syntax(),
                            text,
                            range,
                            objtype,
                            ch,
                            count,
                        )
                    });
                    doc.set_selection(view.id, selection);
                    match op {
                        Operation::Change => change_selection(cx),
                        Operation::Delete => delete_selection(cx),
                        Operation::Yank => yank(cx),
                        Operation::Select => select_mode(cx),
                    }
                }
            }
        }
    });

    let title = match objtype {
        textobject::TextObject::Inside => "Match inside",
        textobject::TextObject::Around => "Match around",
        _ => return,
    };
    let help_text = [
        ("w", "Word"),
        ("W", "WORD"),
        ("p", "Paragraph"),
        ("t", "Type definition (tree-sitter)"),
        ("f", "Function (tree-sitter)"),
        ("a", "Argument/parameter (tree-sitter)"),
        ("c", "Comment (tree-sitter)"),
        ("T", "Test (tree-sitter)"),
        ("e", "Data structure entry (tree-sitter)"),
        ("m", "Closest surrounding pair (tree-sitter)"),
        ("g", "Change"),
        (" ", "... or any character acting as a pair"),
    ];

    cx.editor.autoinfo = Some(Info::new(title, &help_text));
}

pub(crate) fn netrw(cx: &mut Context) {
    let (_, doc) = current!(cx.editor);
    let doc_path = doc.path();
    let mut cwd: PathBuf = PathBuf::new();
    if let Some(path) = doc_path {
        cwd = path.clone();
        cwd.pop();
    } else {
        cwd = current_working_dir();
    }
    let read = cwd.read_dir();
    if let Ok(dirs) = read {
        let mut dir_str = "../\n".to_owned();
        let mut directories: Vec<OsString> = Vec::new();
        let mut files: Vec<OsString> = Vec::new();
        let mut dir_strs: Vec<String> = Vec::new();
        let mut f_strs: Vec<String> = Vec::new();
        for dir in dirs {
            if let Ok(d) = dir {
                let is_dir = d.file_type().unwrap().is_dir();
                if is_dir {
                    directories.push(d.file_name());
                } else {
                    files.push(d.file_name());
                }
            }
        }
        for dir in directories {
            if let Ok(name) = dir.into_string() {
                dir_strs.push(format!("{name}/\n"));
            }
        }
        for file in files {
            if let Ok(name) = file.into_string() {
                f_strs.push(format!("{name}\n"));
            }
        }
        dir_strs.sort();
        f_strs.sort();
        for dir in dir_strs {
            dir_str.push_str(dir.as_str());
        }
        for file in f_strs {
            dir_str.push_str(file.as_str());
        }
        let r = Rope::from(dir_str);
        let mut doc = Document::from(r, None, Arc::new(ArcSwap::new(Arc::new(Config::default()))));
        doc.set_path(Some(cwd.as_path()));
        cx.editor.new_file_from_document(Action::Replace, doc);
        enter_netrw_mode(cx);
    } else {
        log::info!("unable to read dir");
    }
}

pub(crate) fn netrw_open_dir(cx: &mut Context, path: &PathBuf) {
    let cwd = path.clone();
    let read = cwd.read_dir();
    if let Ok(dirs) = read {
        log::info!("{:?}", dirs);
        let mut dir_str = "".to_owned();
        let mut directories: Vec<OsString> = Vec::new();
        let mut files: Vec<OsString> = Vec::new();
        let mut dir_strs: Vec<String> = Vec::new();
        let mut f_strs: Vec<String> = Vec::new();
        for dir in dirs {
            if let Ok(d) = dir {
                let is_dir = d.file_type().unwrap().is_dir();
                if is_dir {
                    directories.push(d.file_name());
                } else {
                    files.push(d.file_name());
                }
            }
        }
        for dir in directories {
            if let Ok(name) = dir.into_string() {
                dir_strs.push(format!("{name}/\n"));
            }
        }
        for file in files {
            if let Ok(name) = file.into_string() {
                f_strs.push(format!("{name}\n"));
            }
        }
        dir_strs.sort();
        f_strs.sort();
        dir_str.push_str("../\n");
        for dir in dir_strs {
            log::info!("dir: {}", dir);
            dir_str.push_str(dir.as_str());
        }
        for file in f_strs {
            log::info!("file: {}", file);
            dir_str.push_str(file.as_str());
        }
        let r = Rope::from(dir_str);
        let mut doc = Document::from(r, None, Arc::new(ArcSwap::new(Arc::new(Config::default()))));
        doc.set_path(Some(cwd.as_path()));
        cx.editor.new_file_from_document(Action::Replace, doc);
        enter_netrw_mode(cx);
    } else {
        log::info!("unable to read dir");
    }
}

pub(crate) fn open_netrw(cx: &mut Context) {
    let mut line_text = String::new();
    let mut current_line = 0;
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    doc.selection(view.id).clone().transform(|range| {
        let pos = range.cursor(text);
        current_line = text.char_to_line(pos);
        range
    });
    let line = text.get_line(current_line);
    if let Some(l) = line {
        line_text = l.as_str().unwrap().to_string()
    }
    let doc_path = doc.path().clone().unwrap().to_str().unwrap();
    let full_path = format!("{}/{}", doc_path, line_text);
    log::info!("line text: {}", line_text);
    log::info!("full_path: {}", full_path);

    if line_text.contains("/") {
        log::info!("opening dir {full_path}");
        let mut path = full_path.chars();
        path.next_back();
        if full_path.contains("..") {
            path.next_back();
            path.next_back();
            path.next_back();
        }
        log::info!("opening dir {}", path.as_str());
        let mut buf = PathBuf::from(path.as_str());
        if full_path.contains("..") {
            buf.pop();
        }
        // netrw_open_dir(cx, &buf);
        let cwd = buf.clone();
        let read = cwd.read_dir();
        if let Ok(dirs) = read {
            log::info!("{:?}", dirs);
            let mut dir_str = "../\n".to_owned();
            let mut directories: Vec<OsString> = Vec::new();
            let mut files: Vec<OsString> = Vec::new();
            let mut dir_strs: Vec<String> = Vec::new();
            let mut f_strs: Vec<String> = Vec::new();
            for dir in dirs {
                if let Ok(d) = dir {
                    let is_dir = d.file_type().unwrap().is_dir();
                    if is_dir {
                        directories.push(d.file_name());
                    } else {
                        files.push(d.file_name());
                    }
                }
            }
            for dir in directories {
                if let Ok(name) = dir.into_string() {
                    dir_strs.push(format!("{name}/\n"));
                }
            }
            for file in files {
                if let Ok(name) = file.into_string() {
                    f_strs.push(format!("{name}\n"));
                }
            }
            dir_strs.sort();
            f_strs.sort();
            for dir in dir_strs {
                log::info!("dir: {}", dir);
                dir_str.push_str(dir.as_str());
            }
            for file in f_strs {
                log::info!("file: {}", file);
                dir_str.push_str(file.as_str());
            }
            let r = Rope::from(dir_str);
            let mut doc =
                Document::from(r, None, Arc::new(ArcSwap::new(Arc::new(Config::default()))));
            doc.set_path(Some(cwd.as_path()));
            let _ = cx.editor.close_document(doc!(cx.editor).id(), true);
            cx.editor.new_file_from_document(Action::Replace, doc);
            enter_netrw_mode(cx);
        } else {
            log::info!("unable to read dir");
        }
    } else {
        log::info!("opening file {full_path}");
        // let buf = PathBuf::from(full_path);
        // let path = buf.as_path();
        let path = Path::new(full_path.trim());
        log::info!("opening path {:?}", path);
        let _ = cx.editor.close_document(doc!(cx.editor).id(), true);
        if let Err(_) = cx.editor.open(&path, Action::Replace) {
            let err = format!("unable to open \"{}\"", path.display());
            cx.editor.set_error(err);
        }
        // cx.editor.close(view!(cx.editor).id);
    }
}

pub(crate) fn netrw_parent_dir(cx: &mut Context) {
    let (_, doc) = current!(cx.editor);
    let doc_path = doc.path().clone().unwrap().to_str().unwrap();
    let mut buf = PathBuf::from(doc_path);
    buf.pop();
    let cwd = buf.clone();
    let read = cwd.read_dir();
    if let Ok(dirs) = read {
        log::info!("{:?}", dirs);
        let mut dir_str = "../\n".to_owned();
        let mut directories: Vec<OsString> = Vec::new();
        let mut files: Vec<OsString> = Vec::new();
        let mut dir_strs: Vec<String> = Vec::new();
        let mut f_strs: Vec<String> = Vec::new();
        for dir in dirs {
            if let Ok(d) = dir {
                let is_dir = d.file_type().unwrap().is_dir();
                if is_dir {
                    directories.push(d.file_name());
                } else {
                    files.push(d.file_name());
                }
            }
        }
        for dir in directories {
            if let Ok(name) = dir.into_string() {
                dir_strs.push(format!("{name}/\n"));
            }
        }
        for file in files {
            if let Ok(name) = file.into_string() {
                f_strs.push(format!("{name}\n"));
            }
        }
        dir_strs.sort();
        f_strs.sort();
        for dir in dir_strs {
            log::info!("dir: {}", dir);
            dir_str.push_str(dir.as_str());
        }
        for file in f_strs {
            log::info!("file: {}", file);
            dir_str.push_str(file.as_str());
        }
        let r = Rope::from(dir_str);
        let mut doc = Document::from(r, None, Arc::new(ArcSwap::new(Arc::new(Config::default()))));
        doc.set_path(Some(cwd.as_path()));
        let _ = cx.editor.close_document(doc!(cx.editor).id(), true);
        cx.editor.new_file_from_document(Action::Replace, doc);
        enter_netrw_mode(cx);
    } else {
        log::info!("unable to read dir");
    }
}

pub(crate) fn get_line_text(cx: &mut Context) -> String {
    let mut current_line = 0;
    let (view, doc) = current!(cx.editor);
    let text = doc.text().slice(..);
    doc.selection(view.id).clone().transform(|range| {
        let pos = range.cursor(text);
        current_line = text.char_to_line(pos);
        range
    });
    let line = text.get_line(current_line);
    if let Some(l) = line {
        return l.as_str().unwrap().to_string();
    }
    String::from("")
}
