use std::io::{self, Read};

const DEBUG_ENABLED: bool = false;

fn main() -> io::Result<()> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    compile(input);
    Ok(())
}

struct State<'a> {
    text: &'a[u8],
    pos: usize,
    next_label: u32
}

impl<'a> State<'a> {
    fn consume_str(&mut self, str: &str) {
        debug(format!("Consuming chars: {}", str));

        for c in str.chars() {
            if self.read_current_char() == c {
                self.pos += 1;
            } else {
                self.parse_error(&format!("Failed consumed string did not match: {}", str)[..])
            }
        }
    }

    fn read_identifier(&mut self) -> String {
        debug(format!("reading next identifier"));
        let mut identifier = String::new();

        if !self.read_current_char().is_ascii_alphabetic() {
            self.parse_error("Unable to find identifier");
        } else {
            identifier.push(self.read_current_char());
        }

        while self.read_next_char().is_ascii_alphanumeric() {
            identifier.push(self.read_current_char());
        }

        debug(format!("Found identifier: {}", identifier));

        return identifier;
    }

    fn consume_whitespace(&mut self) {
        debug(format!("Consuming white space"));

        while " \t\n\r".contains(self.read_current_char()) {
            self.pos += 1;
        }
    }

    fn read_next_char(&mut self) -> char {
        self.pos += 1;
        self.text[self.pos] as char
    }

    fn read_current_char(&self) -> char {
        self.text.get(self.pos).unwrap_or(&0).clone() as char
    }

    fn read_char(&self, pos: usize) -> char {
        self.text[pos] as char
    }

    fn parse_error(&self, message: &str) {
        panic!(format!("{} at position {}", message, self.pos));
    }

    fn new_label(&mut self) -> String {
        let char = ((self.next_label / 100) as u8 + 65) as char;
        let num = self.next_label % 100;
        self.next_label += 1;

        return format!("{}{}", char, num);
    }

    fn read_current_id(&self) -> String {
        let mut id = String::new();
        let mut offset: usize = 0;
        let mut char = self.read_char(self.pos);

        while char.is_ascii_alphanumeric() || char == '.' || char == ',' {
            id.push(char);
            offset += 1;
            char = self.read_char(self.pos + offset);
        }

        debug(format!("Got the ID: {}", id));

        return id;
    }

    fn consume_string(&mut self) -> String {
        debug(format!("Consuming string"));
        let mut string = String::from("'");

        while self.read_next_char() != '\'' {
            string.push(self.read_current_char());
        }

        self.read_next_char();
        string.push('\'');

        debug(format!("Found string: {}", string));

        return string;
    }

    fn read_next_10(&self) -> String {
        let len = if self.pos + 10 >= self.text.len() {
            self.text.len() - self.pos - 1
        }
        else {
            10
        };

        String::from_utf8(self.text[self.pos..self.pos+len].to_vec()).unwrap_or(String::new())
    }
}

fn compile(input: String){
    let mut state = State {
        text: input.as_bytes(),
        pos: 0,
        next_label: 0
    };

    program(&mut state);
}

fn program(state: &mut State) {
    state.consume_str(".SYNTAX");
    state.consume_whitespace();

    print_instruction(format!("ADR {}", state.read_identifier()));
    state.consume_whitespace();


    while state.read_current_char().is_ascii_alphanumeric() {
        st(state);
    }

    state.consume_str(".END");
    print_instruction(format!("END"));
}

fn st(state: &mut State) {
    print_label(state.read_identifier());
    state.consume_whitespace();

    state.consume_str("=");
    state.consume_whitespace();

    ex1(state);

    state.consume_str(".,");
    state.consume_whitespace();

    print_instruction("R".to_string())
}

fn ex1(state: &mut State) {
    state.consume_whitespace();
    let label = state.new_label();

    ex2(state);

    while state .read_current_char() == '/' {
        state.consume_str("/");
        state.consume_whitespace();
        print_instruction(format!("BT {}", label));
        ex2(state);
    }

    state.consume_whitespace();
    print_label(label);
}

fn ex2(state: &mut State) {
    debug(format!("ex2 current car: {}", state.read_next_10()));

    let label = state.new_label();
    let mut end_statement = false;

    match state.read_current_char() {
        '.' => match &state.read_current_id()[..] {
            ".OUT" | ".LABEL" => output(state),
            ".EMPTY" | ".STRING" | ".ID" | ".NUMBER" => {
                ex3(state);
                print_instruction(format!("BF {}", label));
            }
            _ => state.parse_error("Expected identifier")
        },
        _ => {
            let c = state.read_current_char();
            if c.is_ascii_alphabetic() || "\'($".contains(c) {
                ex3(state);
                print_instruction(format!("BF {}", label));
            } else {
                state.parse_error("Expected Symbol")
            }
        }
    }
    debug(format!("here {}", state.read_next_10()));

    while !end_statement && (
        state.read_current_char().is_ascii_alphabetic() ||
            "\'($.".contains(state.read_current_char())
    ) {
        debug(format!("looping {}", state.read_next_10()));
        match state.read_current_char() {
            '.' => match &state.read_current_id()[..] {
                ".," => end_statement = true,
                ".OUT" | ".LABEL" => output(state),
                ".EMPTY" | ".STRING" | ".ID" | ".NUMBER" => {
                    ex3(state);
                    print_instruction(format!("BE"));
                }
                _ => state.parse_error("Expected identifier")
            },
            _ => {
                let c = state.read_current_char();
                if c.is_ascii_alphabetic() || "\'($".contains(c) {
                    ex3(state);
                    print_instruction(format!("BE"));
                } else {
                    state.parse_error("Expected Symbol")
                }
            }
        }
        state.consume_whitespace();
        debug(format!("next char {}",  state.read_current_char()));
    }

    print_label(label);
}

fn ex3(state: &mut State) {
    debug(format!("ex3 current car: {}", state.read_next_10()));

    state.consume_whitespace();

    match state.read_current_char() {
        '.' => match &state.read_current_id()[..] {
                ".EMPTY"=> {
                    state.consume_str(".EMPTY");
                    state.consume_whitespace();
                    print_instruction(format!("SET"));
                },
                ".ID"=> {
                    state.consume_str(".ID");
                    state.consume_whitespace();
                    print_instruction(format!("ID"));
                },
                ".NUMBER"=> {
                    state.consume_str(".NUMBER");
                    state.consume_whitespace();
                    print_instruction(format!("NUM"));
                },
                ".STRING"=> {
                    state.consume_str(".STRING");
                    state.consume_whitespace();
                    print_instruction(format!("STR"));
                },
                _ => state.parse_error("Unknown symbol")
        }
        '(' => {
            state.consume_str("(");
            ex1(state);
            state.consume_str(")");
            state.consume_whitespace();
        },
        '$' => {
            state.consume_str("$");
            state.consume_whitespace();

            let label = state.new_label();
            print_label(format!("{}", label));
            ex3(state);
            print_instruction(format!("BT {}", label));
            print_instruction(format!("SET"));
        },
        '\'' => {
            print_instruction(format!("TST {}", state.consume_string()));
            state.consume_whitespace();
        },
        _ => {
            if state.read_current_char().is_ascii_alphabetic() {
                print_instruction(format!("CLL {}", state.read_identifier()));
                state.consume_whitespace();
            }
            else {
                state.parse_error("identifier or string or symbol (not .OUT nor .LABEL) or $ or (")
            }
        }
    }
    debug(format!("Exit ex3"));
}

fn output(state: &mut State) {
    match &state.read_current_id()[..] {
        ".OUT"=> {
            state.consume_str(".OUT(");
            state.consume_whitespace();
            while state.read_current_char() != ')' {
                out(state);
            }
            state.consume_str(")");

        },
        ".LABEL"=> {
            state.consume_str(".LABEL");
            state.consume_whitespace();
            print_instruction(format!("LB"));
            out(state);
        },
        _ => state.parse_error("Unknown symbol")
    }

    print_instruction(format!("OUT"));
}

fn out(state: &mut State) {
    match state.read_current_char() {
        '*' => {
            state.consume_str("*");
            match state.read_current_char() {
                '1' => {
                    state.consume_str("1");
                    print_instruction(format!("GN1"));
                },
                '2' => {
                    state.consume_str("2");
                    print_instruction(format!("GN2"));
                },
                _ => print_instruction(format!("CI"))
            }
        },
        '\'' => {
            print_instruction(format!("CL {}", state.consume_string()));
            state.consume_whitespace();
        },
        _ => state.parse_error("Expected * or \'")
    }
    state.consume_whitespace();
}

fn print_instruction(instruction: String) {
    println!("\t{}", instruction);
}

fn print_label(label: String) {
    println!("{}", label);
}

fn debug(message: String) {
    if DEBUG_ENABLED {
        println!("{}", message);
    }
}