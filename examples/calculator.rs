use std::num::ParseFloatError;
use std::str::FromStr;

use kas::event::{Command, VirtualKeyCode as VK};
use kas::prelude::*;
use kas::widgets::{EditBox, TextButton};

#[derive(Clone, Debug)]
enum Key {
    Clear,
    DelBack,
    Divide,
    Multiply,
    Subtract,
    Add,
    Equals,
    Char(char),
}

impl_scope! {
    // Buttons get keyboard bindings through the "&" item (e.g. "&1"
    // binds both main and numpad 1 key) and via `with_keys`.
    #[widget{
        layout = grid: {
            0, 0: TextButton::new_msg("&clear", Key::Clear).with_keys(&[VK::Delete]);
            1, 0: TextButton::new_msg("&÷", Key::Divide).with_keys(&[VK::Slash]);
            2, 0: TextButton::new_msg("&×", Key::Multiply).with_keys(&[VK::Asterisk]);
            3, 0: TextButton::new_msg("&−", Key::Subtract);
            0, 1: TextButton::new_msg("&7", Key::Char('7'));
            1, 1: TextButton::new_msg("&8", Key::Char('8'));
            2, 1: TextButton::new_msg("&9", Key::Char('9'));
            3, 1..3: TextButton::new_msg("&+", Key::Add);
            0, 2: TextButton::new_msg("&4", Key::Char('4'));
            1, 2: TextButton::new_msg("&5", Key::Char('5'));
            2, 2: TextButton::new_msg("&6", Key::Char('6'));
            0, 3: TextButton::new_msg("&1", Key::Char('1'));
            1, 3: TextButton::new_msg("&2", Key::Char('2'));
            2, 3: TextButton::new_msg("&3", Key::Char('3'));
            3, 3..5:  TextButton::new_msg("&=", Key::Equals)
                .with_keys(&[VK::Return, VK::NumpadEnter]);
            0..2, 4: TextButton::new_msg("&0", Key::Char('0'));
            2, 4: TextButton::new_msg("&.", Key::Char('.'));
        };
    }]
    #[derive(Debug, Default)]
    struct Buttons(widget_core!());
}

impl_scope! {
    #[impl_default]
    #[widget{
        layout = column: [
            self.display,
            Buttons::default(),
        ];
    }]
    #[derive(Debug)]
    struct CalcUI {
        core: widget_core!(),
        #[widget] display: EditBox = EditBox::new("0")
            .with_editable(false)
            .with_multi_line(true)
            .with_lines(3, 3)
            .with_width_em(5.0, 10.0),
        calc: Calculator = Calculator::new(),
    }
    impl Widget for Self {
        fn configure(&mut self, mgr: &mut ConfigMgr) {
            mgr.disable_nav_focus(true);

            // Enable key bindings without Alt held:
            mgr.enable_alt_bypass(self.id_ref(), true);

            mgr.register_nav_fallback(self.id());
        }

        fn handle_event(&mut self, mgr: &mut EventMgr, event: Event) -> Response {
            match event {
                Event::Command(Command::DelBack) => {
                    if self.calc.handle(Key::DelBack) {
                        *mgr |= self.display.set_string(self.calc.display());
                    }
                    Response::Used
                }
                _ => Response::Unused,
            }
        }

        fn handle_message(&mut self, mgr: &mut EventMgr, _: usize) {
            if let Some(msg) = mgr.try_pop_msg::<Key>() {
                if self.calc.handle(msg) {
                    *mgr |= self.display.set_string(self.calc.display());
                }
            }
        }
    }
    impl Window for Self {
        fn title(&self) -> &str { "Calculator" }
    }
}

fn main() -> kas::shell::Result<()> {
    env_logger::init();

    let theme = kas::theme::ShadedTheme::new().with_font_size(16.0);
    kas::shell::Toolkit::new(theme)?
        .with(CalcUI::default())?
        .run()
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Op {
    None,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Clone, Debug)]
struct Calculator {
    state: Result<f64, ParseFloatError>,
    op: Op,
    line_buf: String,
}

impl Calculator {
    fn new() -> Calculator {
        Calculator {
            state: Ok(0.0),
            op: Op::None,
            line_buf: String::new(),
        }
    }

    fn state_str(&self) -> String {
        match &self.state {
            Ok(x) => x.to_string(),
            Err(e) => format!("{}", e),
        }
    }

    fn display(&self) -> String {
        let op = match self.op {
            Op::None => "",
            Op::Add => "+",
            Op::Subtract => "−",
            Op::Multiply => "×",
            Op::Divide => "÷",
        };
        format!("{}\n{}\n{}", self.state_str(), op, &self.line_buf)
    }

    // return true if display changes
    fn handle(&mut self, key: Key) -> bool {
        match key {
            Key::Clear => {
                self.state = Ok(0.0);
                self.op = Op::None;
                self.line_buf.clear();
                true
            }
            Key::DelBack => self.line_buf.pop().is_some(),
            Key::Divide => self.do_op(Op::Divide),
            Key::Multiply => self.do_op(Op::Multiply),
            Key::Subtract => self.do_op(Op::Subtract),
            Key::Add => self.do_op(Op::Add),
            Key::Equals => self.do_op(Op::None),
            Key::Char(c) => {
                self.line_buf.push(c);
                true
            }
        }
    }

    fn do_op(&mut self, next_op: Op) -> bool {
        if self.line_buf.is_empty() {
            self.op = next_op;
            return true;
        }

        let line = f64::from_str(&self.line_buf);
        self.line_buf.clear();

        if self.op == Op::None {
            self.state = line;
        } else if let Ok(x) = self.state {
            self.state = match line {
                Ok(y) => {
                    match self.op {
                        Op::Divide => Ok(x / y),
                        Op::Multiply => Ok(x * y),
                        Op::Subtract => Ok(x - y),
                        Op::Add => Ok(x + y),
                        _ => panic!("unexpected op"), // program error
                    }
                }
                e @ Err(_) => e,
            };
        }

        self.op = next_op;
        true
    }
}
