use async_trait::async_trait;

use crate::descriptor::{ParameterType, ToolDescriptor, ToolParameter};
use crate::executor::{ToolCall, ToolExecutor, ToolResult};

pub struct CalculatorTool {
    descriptor: ToolDescriptor,
}

impl Default for CalculatorTool {
    fn default() -> Self {
        Self::new()
    }
}

impl CalculatorTool {
    pub fn new() -> Self {
        Self {
            descriptor: ToolDescriptor::new("calculator", "Evaluate mathematical expressions")
                .with_parameters(vec![ToolParameter::required(
                    "expression",
                    ParameterType::String,
                    "Mathematical expression to evaluate",
                )]),
        }
    }
}

#[async_trait]
impl ToolExecutor for CalculatorTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn execute(&self, call: ToolCall) -> ToolResult {
        let expression = match call.arguments.get("expression").and_then(|v| v.as_str()) {
            Some(e) => e,
            None => return ToolResult::error(&call, "missing required parameter: expression"),
        };

        let mut parser = Parser::new(expression);
        match parser.parse_expr() {
            Ok(result) => ToolResult::success(&call, format!("{result}")),
            Err(e) => ToolResult::error(&call, e),
        }
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Self {
        Self {
            chars: input.chars().collect(),
            pos: 0,
        }
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.chars.get(self.pos).copied();
        self.pos += 1;
        ch
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    fn parse_expr(&mut self) -> Result<f64, String> {
        let mut result = self.parse_term()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('+') => {
                    self.advance();
                    result += self.parse_term()?;
                }
                Some('-') => {
                    self.advance();
                    result -= self.parse_term()?;
                }
                _ => break,
            }
        }
        Ok(result)
    }

    fn parse_term(&mut self) -> Result<f64, String> {
        let mut result = self.parse_factor()?;
        loop {
            self.skip_whitespace();
            match self.peek() {
                Some('*') => {
                    self.advance();
                    result *= self.parse_factor()?;
                }
                Some('/') => {
                    self.advance();
                    let divisor = self.parse_factor()?;
                    if divisor == 0.0 {
                        return Err("division by zero".to_string());
                    }
                    result /= divisor;
                }
                _ => break,
            }
        }
        Ok(result)
    }

    fn parse_factor(&mut self) -> Result<f64, String> {
        self.skip_whitespace();
        match self.peek() {
            Some('(') => {
                self.advance();
                let result = self.parse_expr()?;
                self.skip_whitespace();
                if self.peek() != Some(')') {
                    return Err("expected ')'".to_string());
                }
                self.advance();
                Ok(result)
            }
            Some('-') => {
                self.advance();
                Ok(-self.parse_factor()?)
            }
            Some('+') => {
                self.advance();
                self.parse_factor()
            }
            Some(ch) if ch.is_ascii_digit() || ch == '.' => {
                let mut num_str = String::new();
                while let Some(ch) = self.peek() {
                    if ch.is_ascii_digit() || ch == '.' {
                        num_str.push(ch);
                        self.advance();
                    } else {
                        break;
                    }
                }
                num_str
                    .parse::<f64>()
                    .map_err(|e| format!("invalid number: {e}"))
            }
            Some(ch) => Err(format!("unexpected character: '{ch}'")),
            None => Err("unexpected end of expression".to_string()),
        }
    }
}
