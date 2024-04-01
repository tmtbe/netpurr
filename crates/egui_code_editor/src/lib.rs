use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use egui::{Align, Area, Context, Event, EventFilter, Frame, Id, Key, Layout, Order, Pos2, pos2, Response, RichText, TextBuffer, Ui};
use egui::text::{CCursor, CCursorRange, CursorRange};
use egui::text_edit::TextEditState;
#[cfg(feature = "egui")]
use egui::widgets::text_edit::TextEditOutput;
use serde::{Deserialize, Serialize};

#[cfg(feature = "egui")]
use highlighting::highlight;
pub use highlighting::Token;
pub use syntax::{Syntax, TokenType};
pub use themes::ColorTheme;
pub use themes::DEFAULT_THEMES;

pub mod highlighting;
mod syntax;
#[cfg(test)]
mod tests;
mod themes;

#[derive(Clone, Debug, PartialEq)]
/// CodeEditor struct which stores settings for highlighting.
pub struct CodeEditor {
    id: String,
    theme: ColorTheme,
    syntax: Syntax,
    numlines: bool,
    fontsize: f32,
    rows: usize,
    vscroll: bool,
    stick_to_bottom: bool,
    shrink: bool,
    _prompt: Prompt,
    _popup_id:Id,
}

#[derive(Clone, Debug, PartialEq, Default,Serialize,Deserialize)]
pub struct Prompt {
    pub map: BTreeMap<String,PromptInfo>
}
#[derive(Clone, Debug, PartialEq, Default,Serialize,Deserialize)]
pub struct PromptInfo{
    pub desc:String,
    pub fill:String
}
impl Prompt{
    pub fn from_str(text:&str) ->Self{
        serde_yaml::from_str(text).unwrap_or_default()
    }
}

impl Hash for CodeEditor {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.theme.hash(state);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        (self.fontsize as u32).hash(state);
        self.syntax.hash(state);
    }
}

impl Default for CodeEditor {
    fn default() -> CodeEditor {
        CodeEditor {
            id: String::from("Code Editor"),
            theme: ColorTheme::GRUVBOX,
            syntax: Syntax::rust(),
            numlines: true,
            fontsize: 10.0,
            rows: 10,
            vscroll: true,
            stick_to_bottom: false,
            shrink: false,
            _prompt: Prompt::default(),
            _popup_id: Id::new("code_editor_prompt"),
        }
    }
}

impl CodeEditor {
    pub fn id_source(self, id_source: impl Into<String>) -> Self {
        let id =  id_source.into();
        CodeEditor {
            id: id.clone(),
            _popup_id: Id::new(format!("{}_{}",id.clone(),"popup")),
            ..self
        }
    }

    /// Minimum number of rows to show.
    ///
    /// **Default: 10**
    pub fn with_rows(self, rows: usize) -> Self {
        CodeEditor { rows, ..self }
    }

    /// Use custom Color Theme
    ///
    /// **Default: Gruvbox**
    pub fn with_theme(self, theme: ColorTheme) -> Self {
        CodeEditor { theme, ..self }
    }

    /// Use custom font size
    ///
    /// **Default: 10.0**
    pub fn with_fontsize(self, fontsize: f32) -> Self {
        CodeEditor { fontsize, ..self }
    }

    #[cfg(feature = "egui")]
    /// Use UI font size
    pub fn with_ui_fontsize(self, ui: &mut egui::Ui) -> Self {
        CodeEditor {
            fontsize: egui::TextStyle::Monospace.resolve(ui.style()).size,
            ..self
        }
    }

    /// Show or hide lines numbering
    ///
    /// **Default: true**
    pub fn with_numlines(self, numlines: bool) -> Self {
        CodeEditor { numlines, ..self }
    }

    /// Use custom syntax for highlighting
    ///
    /// **Default: Rust**
    pub fn with_syntax(self, syntax: Syntax) -> Self {
        CodeEditor { syntax, ..self }
    }

    pub fn with_prompt(self, prompt: Prompt) -> Self {
        CodeEditor { _prompt: prompt, ..self }
    }

    /// Turn on/off scrolling on the vertical axis.
    ///
    /// **Default: true**
    pub fn vscroll(self, vscroll: bool) -> Self {
        CodeEditor { vscroll, ..self }
    }
    /// Should the containing area shrink if the content is small?
    ///
    /// **Default: false**
    pub fn auto_shrink(self, shrink: bool) -> Self {
        CodeEditor { shrink, ..self }
    }

    /// Stick to bottom
    /// The scroll handle will stick to the bottom position even while the content size
    /// changes dynamically. This can be useful to simulate terminal UIs or log/info scrollers.
    /// The scroll handle remains stuck until user manually changes position. Once "unstuck"
    /// it will remain focused on whatever content viewport the user left it on. If the scroll
    /// handle is dragged to the bottom it will again become stuck and remain there until manually
    /// pulled from the end position.
    ///
    /// **Default: false**
    pub fn stick_to_bottom(self, stick_to_bottom: bool) -> Self {
        CodeEditor {
            stick_to_bottom,
            ..self
        }
    }

    #[cfg(feature = "egui")]
    pub fn format(&self, ty: TokenType) -> egui::text::TextFormat {
        let font_id = egui::FontId::monospace(self.fontsize);
        let color = self.theme.type_color(ty);
        egui::text::TextFormat::simple(font_id, color)
    }

    #[cfg(feature = "egui")]
    fn numlines_show(&self, ui: &mut egui::Ui, text: &str) {
        let total = if text.ends_with('\n') || text.is_empty() {
            text.lines().count() + 1
        } else {
            text.lines().count()
        }
        .max(self.rows);
        let max_indent = total.to_string().len();
        let mut counter = (1..=total)
            .map(|i| {
                let label = i.to_string();
                format!(
                    "{}{label}",
                    " ".repeat(max_indent.saturating_sub(label.len()))
                )
            })
            .collect::<Vec<String>>()
            .join("\n");

        #[allow(clippy::cast_precision_loss)]
        let width = max_indent as f32 * self.fontsize * 0.5;

        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
            let layout_job = egui::text::LayoutJob::single_section(
                string.to_string(),
                egui::TextFormat::simple(
                    egui::FontId::monospace(self.fontsize),
                    self.theme.type_color(TokenType::Comment(true)),
                ),
            );
            ui.fonts(|f| f.layout_job(layout_job))
        };

        ui.add(
            egui::TextEdit::multiline(&mut counter)
                .id_source(format!("{}_numlines", self.id))
                .font(egui::TextStyle::Monospace)
                .interactive(false)
                .frame(false)
                .desired_rows(self.rows)
                .desired_width(width)
                .layouter(&mut layouter),
        );
    }

    #[cfg(feature = "egui")]
    /// Show Code Editor
    pub fn show(&mut self, ui: &mut egui::Ui, text: &mut String) -> TextEditOutput {
        let mut need_insert_text = false;
        let mut need_up = false;
        let mut need_down = false;
        let mut is_pop = false;
        ui.ctx().memory(|mem|{
            if mem.is_popup_open(self._popup_id) {
                is_pop = true;
            }
        });
        if is_pop{
            ui.input(|input_state|{
                if input_state.key_released(Key::Tab)||input_state.key_pressed(Key::Enter) {
                    need_insert_text = true;
                }
                if input_state.key_released(Key::ArrowUp){
                    need_up = true;
                }
                if input_state.key_released(Key::ArrowDown){
                    need_down = true;
                }
            });
            if need_up{
                let mut prompt_state = CodeEditorPromptState::load(ui.ctx(),self._popup_id);
                let mut index = 0;
                if prompt_state.index<=0{
                    index = prompt_state.prompts.len()-1;
                }else{
                    index = prompt_state.index-1;
                }
                prompt_state.index = index;
                prompt_state.select = prompt_state.prompts.get(index).cloned().unwrap_or_default();
                prompt_state.store(ui.ctx(),self._popup_id);
            }
            if need_down{
                let mut prompt_state = CodeEditorPromptState::load(ui.ctx(),self._popup_id);
                let mut index = prompt_state.index+1;
                if index>=prompt_state.prompts.len(){
                    index = 0;
                }
                prompt_state.index = index;
                prompt_state.select = prompt_state.prompts.get(index).cloned().unwrap_or_default();
                prompt_state.store(ui.ctx(),self._popup_id);
            }
            ui.input_mut(|i|{
                i.events.retain(|e|{
                    match e {
                        Event::Key { key, .. } => {
                            let k = key.clone();
                            if k==Key::ArrowUp||k==Key::ArrowDown||k==Key::Tab||k==Key::Enter{
                                return false
                            }
                            return true
                        }
                        _=>{
                            return true
                        }
                    }
                })
            });
        }

        let mut text_edit_output: Option<TextEditOutput> = None;
        let mut code_editor = |ui: &mut egui::Ui| {
            ui.horizontal_top(|h| {
                self.theme.modify_style(h, self.fontsize);
                if self.numlines {
                    self.numlines_show(h, text);
                }
                egui::ScrollArea::horizontal()
                    .id_source(format!("{}_inner_scroll", self.id))
                    .show(h, |ui| {
                        let mut layouter = |ui: &egui::Ui, string: &str, _wrap_width: f32| {
                            let layout_job = highlight(ui.ctx(), self, string);
                            ui.fonts(|f| f.layout_job(layout_job))
                        };
                        let output = egui::TextEdit::multiline(text)
                            .id_source(&self.id)
                            .lock_focus(true)
                            .desired_rows(self.rows)
                            .frame(true)
                            .desired_width(if self.shrink { 0.0 } else { f32::MAX })
                            .layouter(&mut layouter)
                            .show(ui);
                        self.popup(ui, text,&output);
                        if need_insert_text {
                            let prompt_state = CodeEditorPromptState::load(ui.ctx(),self._popup_id);
                            let fill_text = self._prompt.map.get(prompt_state.select.as_str()).unwrap().fill.clone();
                            prompt_state.cursor_range.map(|c|{
                                text.insert_text(
                                    &fill_text[prompt_state.prompt.len()..].to_string(),
                                    c.primary.ccursor.index,
                                );
                                ui.memory_mut(|mem| mem.toggle_popup(self._popup_id));
                                let mut tes = TextEditState::load(ui.ctx(), output.response.id).unwrap();
                                tes.cursor.set_char_range(Some(CCursorRange {
                                    primary: CCursor {
                                        index: c.primary.ccursor.index+fill_text.len()-prompt_state.prompt.len(),
                                        prefer_next_row: false,
                                    },
                                    secondary: CCursor {
                                        index: c.primary.ccursor.index+fill_text.len()-prompt_state.prompt.len(),
                                        prefer_next_row: false,
                                    },
                                }));
                                tes.store(ui.ctx(), output.response.id);
                                ui.ctx().request_repaint();
                            });
                        }
                        text_edit_output = Some(output);
                    });
            });
        };
        if self.vscroll {
            egui::ScrollArea::vertical()
                .id_source(format!("{}_outer_scroll", self.id))
                .stick_to_bottom(self.stick_to_bottom)
                .show(ui, code_editor);
        } else {
            code_editor(ui);
        }
        text_edit_output.expect("TextEditOutput should exist at this point")
    }
    fn popup(&self, ui: &mut Ui, text: &mut String, output:&TextEditOutput) {
        output.cursor_range.map(|c| {
            let len = text.as_str().len();
            if len <= 1 || c.primary.ccursor.index > len {
                return;
            }
            let before_cursor_text: String = text
                .as_str()
                .chars()
                .take(c.primary.ccursor.index)
                .collect();

            let mut after_cursor_text: String = text.as_str().chars().skip(c.primary.ccursor.index).collect();
            after_cursor_text = after_cursor_text.replace("\n"," ")
                .replace(","," ")
                .replace("\t"," ").replace(";"," ");
            if !after_cursor_text.starts_with(" ")&&!after_cursor_text.is_empty(){
                ui.memory_mut(|mem| {
                    if mem.is_popup_open(self._popup_id) {
                        mem.close_popup()
                    }
                });
                return;
            }
            let (prompt,prompts) = self.find_prompt(before_cursor_text);
            if prompts.len()>0 {
                ui.memory_mut(|mem| {
                    mem.open_popup(self._popup_id)
                });
                self.popup_prompt_widget(
                    ui,
                    pos2(
                        output.galley_pos.x
                            + (c.primary.rcursor.column as f32) * (12.0 / 2.0 + 1.0),
                        output.galley_pos.y
                            + (c.primary.rcursor.row as f32) * (12.0 / 2.0 + 1.0)
                            + 16.0,
                    ),
                    |ui| {
                        self.render_popup_prompt(c, prompt, prompts, ui)
                    },
                );
            }else{
                ui.memory_mut(|mem| {
                    if mem.is_popup_open(self._popup_id) {
                        mem.close_popup()
                    }
                });
            }
        });
    }
    fn render_popup_prompt(
        &self,
        c: CursorRange,
        prompt: String,
        prompts: Vec<String>,
        ui: &mut Ui,
    ) {
        let mut prompt_state = CodeEditorPromptState::load(ui.ctx(),self._popup_id);
        ui.horizontal(|ui| {
            let scroll =   egui::ScrollArea::vertical()
                .max_width(150.0)
                .min_scrolled_height(150.0)
                .max_height(400.0);
            scroll.show(ui, |ui| {
                ui.vertical(|ui| {
                    let mut not_found = false;
                    if prompts.iter().find(|key|{
                        key.as_str()==prompt_state.select
                    }).is_none(){
                        not_found = true;
                    }
                    for (index, key) in prompts.iter().enumerate() {
                        let show_text = key.split(".").last().unwrap_or_default();
                        let label =
                            ui.selectable_value(&mut prompt_state.select ,key.to_string(),RichText::new(show_text).strong());
                        if index == 0 {
                            if not_found{
                                prompt_state.select = key.to_string();
                                prompt_state.index = 0;
                            }
                        }
                        if index==prompt_state.index{
                            label.scroll_to_me(Some(Align::Center));
                        }
                    }
                    let new_prompt_state = CodeEditorPromptState{
                        prompts:prompts.clone(),
                        select:prompt_state.select.clone(),
                        index:prompts.iter().position(|x|x.as_str()==prompt_state.select.as_str()).unwrap(),
                        prompt:prompt.clone(),
                        cursor_range: Some(c.clone()),
                    };
                    new_prompt_state.store(ui.ctx(),self._popup_id);
                });
            });
            ui.separator();
            ui.vertical(|ui| {
                let prompt_state = CodeEditorPromptState::load(ui.ctx(),self._popup_id);
                ui.heading(prompt_state.select.as_str());
                ui.horizontal_wrapped(|ui|{
                    ui.strong(self._prompt.map.get(prompt_state.select.as_str()).cloned().unwrap_or_default().desc)
                });
            });
        });
    }

    fn popup_prompt_widget<R>(
        &self,
        ui: &Ui,
        suggested_position: Pos2,
        add_contents: impl FnOnce(&mut Ui) -> R,
    ) -> Option<R> {
        if ui.memory(|mem| mem.is_popup_open(self._popup_id)) {
            let inner = Area::new(self._popup_id)
                .order(Order::Foreground)
                .constrain(true)
                .fixed_pos(suggested_position)
                .show(ui.ctx(), |ui| {
                    let frame = Frame::popup(ui.style());
                    frame
                        .show(ui, |ui| {
                            ui.with_layout(Layout::left_to_right(Align::LEFT), |ui| add_contents(ui))
                                .inner
                        })
                        .inner
                })
                .inner;
            Some(inner)
        } else {
            None
        }
    }
    fn count_dots(sentence: &str) -> usize {
        let dot: char = '.';
        sentence.chars().filter(|&c| c == dot).count()
    }
    fn find_prompt(&self,text:String)->(String,Vec<String>){
        let mut replace = text
            .replace("{"," ")
            .replace("}"," ")
            .replace(";"," ")
            .replace(","," ")
            .replace("\t"," ")
            .replace("\n"," ");
        if text.ends_with("("){
            replace = replace.replace("("," ");
        }
        let sentence = replace.split(" ").last().unwrap_or_default();
        if sentence.is_empty(){
            return ("".to_string(),vec![])
        }
        let dot_count = Self::count_dots(sentence);
        (sentence.to_string(), self._prompt.map.keys().filter(|(k)|{
            k.starts_with(sentence)&&k.as_str()!= sentence
        }).filter(|(k)|{
            Self::count_dots(k)==dot_count
        }).cloned().collect())
    }
}

#[derive(Clone, Default, Serialize, Deserialize, Debug)]
struct CodeEditorPromptState {
    pub select: String,
    pub prompt: String,
    pub prompts:Vec<String>,
    pub index: usize,
    pub cursor_range:Option<CursorRange>
}
impl CodeEditorPromptState {
    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|d| d.get_persisted(id)).unwrap_or_default()
    }
    pub fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|d| d.insert_persisted(id, self));
    }
}