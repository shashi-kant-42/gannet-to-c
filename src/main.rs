use std::{fmt::Debug, marker::PhantomData};

use pa_rs::parser::*;

#[derive(Clone, Copy)]
struct DigitParser;
impl Parse for DigitParser {
    type Result = i64;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        let digits = ('0'..='9')
            .map(|ch| char_p(ch));
        one_of_p(digits)
            .map(|ch| ch.to_digit(16).unwrap() as i64)
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct LetterParser;
impl Parse for LetterParser {
    type Result = char;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        let letters = ('a'..='z')
            .chain('A'..='Z')
            .map(|ch| char_p(ch));
        one_of_p(letters)
        .parse(input)
    }
}

#[derive(Clone, Copy)]
struct SpecialCharParser;
impl Parse for SpecialCharParser {
    type Result = char;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        let letters = ['_', '.', '-', '?', '!', '/', '*', '+', '<', '>', '=']
            .map(|ch| char_p(ch));
        one_of_p(letters)
        .parse(input)
    }
}

#[derive(Clone, Copy)]
struct IntegerParser;
impl Parse for IntegerParser {
    type Result = i64;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        char_p('-')
            .optional()
            .and(DigitParser.one_or_more())
            .map(|(negation, digits)| {
                let mut result = 0;
                for digit in digits {
                    result = 10*result + digit;
                }
                if let Some(_) = negation {
                    result *= -1;
                }
                result
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct FloatParser;
impl Parse for FloatParser {
    type Result = f64;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        char_p('-')
            .optional()
            .and(DigitParser.one_or_more())
            .and(char_p('.').drop(DigitParser.one_or_more()))
            .map(|((negation, bd), ad)| {
                let mut result = 0.;
                for digit in bd {
                    result = 10.*result + digit as f64;
                }
                let mut factor = 0.1;
                for digit in ad {
                    result += factor * digit as f64;
                    factor *= 0.1;
                }
                if let Some(_) = negation {
                    result *= -1.;
                }
                result
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct StringParser;
impl Parse for StringParser {
    type Result = String;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        char_p('"')
            .drop(any_char_except(['"']).zero_or_more())
            .keep(char_p('"'))
            .map(|chars| chars.into_iter().collect())
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct SymbolParser;
impl Parse for SymbolParser {
    type Result = Symbol;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        LetterParser.or(SpecialCharParser)
            .and(LetterParser.or(DigitParser).or(SpecialCharParser).zero_or_more())
            .map(|(head, tail)| {
                let mut symbol = String::new();
                match head {
                    Either::Left(letter) => {
                        symbol.push(letter);
                    },
                    Either::Right(special) => {
                        symbol.push(special);
                    },
                }

                for ch in tail {
                    match ch {
                        Either::Left(letter_or_digit) => {
                            match letter_or_digit {
                                Either::Left(letter) => {
                                    symbol.push(letter);
                                },
                                Either::Right(digit) => {
                                    symbol = symbol + &digit.to_string();
                                }
                            }
                        },
                        Either::Right(special) => {
                            symbol.push(special);
                        }
                    }
                }

                Symbol(symbol)
            })
            .parse(input)
    }
}


#[derive(Debug, PartialEq)]
enum Literal {
    Integer(i64),
    Float(f64),
    String(String),
}

#[derive(Clone, Copy)]
struct LiteralParser;
impl Parse for LiteralParser {
    type Result = Literal;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        IntegerParser
            .or_enum(Literal::Integer)
            .or(FloatParser, Literal::Float)
            .or(StringParser, Literal::String)
            .parse(input)
    }
}

#[derive(Debug, PartialEq)]
enum QuotedExpr {
    Literal(Literal),
    Symbol(Symbol),
    SExpression(SExpression),
}

#[derive(Debug, PartialEq)]
struct UnquotedExpr(Box<Expression>);

#[derive(Debug, PartialEq)]
enum Expression {
    Literal(Literal),
    Symbol(Symbol),
    QuotedExpr(QuotedExpr),
    UnquotedExpr(UnquotedExpr),
    LetExpr(LetExpr),
    BeginExpr(BeginExpr),
    AssignExpr(AssignExpr),
    ConstExpr(ConstExpr),
    ReadExpr(ReadExpr),
    UpdateExpr(UpdateExpr),
    LocalExpr(LocalExpr),
    CountExpr(CountExpr),
    IfExpr(IfExpr),
    LambdaExpr(LambdaExpr),
    ApplyExpr(ApplyExpr),
    LabelExpr(LabelExpr),
    ReturnExpr(ReturnExpr),
    ListExpr(ListExpr),
    HeadExpr(HeadExpr),
    TailExpr(TailExpr),
    LengthExpr(LengthExpr),
    AppendExpr(AppendExpr),
    FopenExpr(FopenExpr),
    FcloseExpr(FcloseExpr),
    IoReadExpr(IoReadExpr),
    IoWriteExpr(IoWriteExpr),
    BufExpr(BufExpr),
    IterExpr(IterExpr),
    RandExpr(RandExpr),
    SExpression(SExpression),
}

#[derive(Debug, PartialEq)]
struct Symbol(String);

#[derive(Debug, PartialEq)]
struct QuotedSymbol(String);

#[derive(Debug, PartialEq)]
enum SymbolOrQuotedSymbol {
    Quoted(QuotedSymbol),
    Unquoted(Symbol),
}

#[derive(Debug, PartialEq)]
struct SExpression {
    symbol: Symbol,
    expression: Vec<Expression>,
}


#[derive(Debug, PartialEq)]
struct BeginExpr {
    expression: Vec<Expression>,
}


#[derive(Debug, PartialEq)]
struct LetExpr {
    scope_item: ScopeItem
}

#[derive(Debug, PartialEq)]
enum ScopeItem {
    AssignExpr(AssignExpr),
    ConstExpr(ConstExpr),
    LocalExpr(LocalExpr),
    ReadExpr(ReadExpr),
    UpdateExpr(UpdateExpr),
    CountExpr(CountExpr),
    Expression(Box<Expression>),
}



#[derive(Debug, PartialEq)]
struct AssignExpr {
    symbol: SymbolOrQuotedSymbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct ConstExpr {
    symbol: Symbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct ReadExpr {
    symbol: SymbolOrQuotedSymbol,
}

#[derive(Debug, PartialEq)]
struct UpdateExpr {
    symbol: SymbolOrQuotedSymbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct LocalExpr {
    symbol: Symbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct CountExpr {
    expression: Vec<Expression>,
}


#[derive(Debug, PartialEq)]
enum ConditionType {
    If,
    IfTc
}

#[derive(Debug, PartialEq)]
struct IfExpr {
    condition_type: ConditionType,
    predicate: Box<Expression>,
    then_branch: Box<Expression>,
    else_branch: Option<Box<Expression>>,
}

#[derive(Debug, PartialEq)]
struct LambdaExpr {
    params: Vec<SymbolOrQuotedSymbol>,
    body: Box<Expression>,
}

#[derive(Debug, PartialEq)]
enum ApplyType {
    Apply,
    ApplyTc,
    ApplyRec,
}

#[derive(Debug, PartialEq)]
struct ApplyExpr {
    apply_type: ApplyType,
    function_ref: Box<Expression>,
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
struct LabelExpr {
    symbol: Symbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
enum ReturnType {
    Return,
    ReturnTc,
}

#[derive(Debug, PartialEq)]
struct ReturnExpr {
    return_type: ReturnType,
    expression: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
struct ListExpr {
    expressions: Vec<Expression>,
}

#[derive(Debug, PartialEq)]
struct HeadExpr {
    list_ref: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct TailExpr {
    list_ref: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct LengthExpr {
    list_ref: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct AppendExpr {
    list_ref1: Box<Expression>,
    list_ref2: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct FopenExpr {
    handle: Box<Expression>,
    filename: String,
    mode: String,
}

#[derive(Debug, PartialEq)]
struct FcloseExpr {
    handle: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct IoReadExpr {
    handle: Box<Expression>,
    offset: Option<Box<Expression>>,
}

#[derive(Debug, PartialEq)]
struct IoWriteExpr {
    handle: Box<Expression>,
    data: Box<Expression>,
}

#[derive(Debug, PartialEq)]
struct BufExpr {
    symbol: Symbol,
    expression: Box<Expression>,
}

#[derive(Debug, PartialEq)]
enum IterExpr {
    IterLoop(Box<Expression>),
    IterValue,
}

#[derive(Debug, PartialEq)]
struct RandExpr {
    expression: Box<Expression>,
}


#[derive(Clone, Copy)]
struct QuotedExprParser;
impl Parse for QuotedExprParser {
    type Result = QuotedExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        char_p('\'')
            .drop(LiteralParser
                .or(SymbolParser)
                .or(SExpressionParser))
            .map(|either| {
                match either {
                    Either::Left(literal_or_symbol) => {
                        match literal_or_symbol {
                            Either::Left(literal) => QuotedExpr::Literal(literal),
                            Either::Right(symbol) => QuotedExpr::Symbol(symbol),
                        }
                    },
                    Either::Right(s_expression) => QuotedExpr::SExpression(s_expression),
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct UnquotedExprParser;
impl Parse for UnquotedExprParser {
    type Result = UnquotedExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
            str_p("unquote").sbws()
            .drop(ExpressionParser)
            .surrounded_by_rb()
            .map(|expression| {
                UnquotedExpr(Box::new(expression))
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ExpressionParser;
impl Parse for ExpressionParser {
    type Result = Expression;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        LiteralParser
            .or_enum(Expression::Literal)
            .or(SymbolParser, Expression::Symbol)
            .or(QuotedExprParser, Expression::QuotedExpr)
            .or(UnquotedExprParser, Expression::UnquotedExpr)
            .or(LetExprParser, Expression::LetExpr)
            .or(BeginExprParser, Expression::BeginExpr)
            .or(AssignExprParser, Expression::AssignExpr)
            .or(ConstExprParser, Expression::ConstExpr)
            .or(ReadExprParser, Expression::ReadExpr)
            .or(UpdateExprParser, Expression::UpdateExpr)
            .or(LocalExprParser, Expression::LocalExpr)
            .or(CountExprParser, Expression::CountExpr)
            .or(IfExprParser, Expression::IfExpr)
            .or(LambdaExprParser, Expression::LambdaExpr)
            .or(ApplyExprParser, Expression::ApplyExpr)
            .or(LabelExprParser, Expression::LabelExpr)
            .or(ReturnExprParser, Expression::ReturnExpr)
            .or(ListExprParser, Expression::ListExpr)
            .or(ReadExprParser, Expression::ReadExpr)
            .or(HeadExprParser, Expression::HeadExpr)
            .or(TailExprParser, Expression::TailExpr)
            .or(LengthExprParser, Expression::LengthExpr)
            .or(AppendExprParser, Expression::AppendExpr)
            .or(FopenExprParser, Expression::FopenExpr)
            .or(FcloseExprParser, Expression::FcloseExpr)
            .or(IoReadExprParser, Expression::IoReadExpr)
            .or(IoWriteExprParser, Expression::IoWriteExpr)
            .or(BufExprParser, Expression::BufExpr)
            .or(IterExprParser, Expression::IterExpr)
            .or(RandExprParser, Expression::RandExpr)
            .or(SExpressionParser, Expression::SExpression)
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct SExpressionParser;
impl Parse for SExpressionParser {
    type Result = SExpression;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
            SymbolParser.sbws().and(ExpressionParser.sbws().zero_or_more())
            .surrounded_by_rb()
            .sbws()
            .map(|(symbol, expression)| {
                SExpression { symbol, expression }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct BeginExprParser;
impl Parse for BeginExprParser {
    type Result = BeginExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("begin").sbws()
            .drop(ExpressionParser.zero_or_more())
            .surrounded_by_rb()
            .map(|expression| {
                BeginExpr {
                    expression
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct QuotedSymbolParser;
impl Parse for QuotedSymbolParser {
    type Result = QuotedSymbol;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        char_p('\'')
            .drop(SymbolParser)
            .map(|symbol| {
                QuotedSymbol(symbol.0)
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct SymbolOrQuotedSymbolParser;
impl Parse for SymbolOrQuotedSymbolParser {
    type Result = SymbolOrQuotedSymbol;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        QuotedSymbolParser
            .or(SymbolParser)
            .map(|quoted_or_symbol| {
                match quoted_or_symbol {
                    Either::Left(quoted_symbol) => SymbolOrQuotedSymbol::Quoted(quoted_symbol),
                    Either::Right(symbol) => SymbolOrQuotedSymbol::Unquoted(symbol),
                }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct AssignExprParser;
impl Parse for AssignExprParser {
    type Result = AssignExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("assign")
            .sbws()
            .drop(SymbolOrQuotedSymbolParser)
            .and(ExpressionParser)
            .sbws()
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                AssignExpr {
                    symbol,
                    expression: Box::new(expression)
                }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct ConstExprParser;
impl Parse for ConstExprParser {
    type Result = ConstExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("const")
            .sbws()
            .drop(SymbolParser)
            .and(ExpressionParser)
            .sbws()
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                ConstExpr {
                    symbol,
                    expression: Box::new(expression),
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ReadExprParser;
impl Parse for ReadExprParser {
    type Result = ReadExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("read")
            .sbws()
            .drop(SymbolOrQuotedSymbolParser.sbws())
            .surrounded_by_rb()
            .map(|symbol| ReadExpr { symbol })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct UpdateExprParser;
impl Parse for UpdateExprParser {
    type Result = UpdateExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("update")
            .sbws()
            .drop(SymbolOrQuotedSymbolParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                UpdateExpr { symbol, expression: Box::new(expression) }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct LocalExprParser;
impl Parse for LocalExprParser {
    type Result = LocalExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("local")
            .sbws()
            .drop(SymbolParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                LocalExpr { symbol, expression: Box::new(expression) }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct CountExprParser;
impl Parse for CountExprParser {
    type Result = CountExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("count")
            .sbws()
            .drop(ExpressionParser.zero_or_more())
            .sbws()
            .surrounded_by_rb()
            .map(|expression| {
                CountExpr { expression }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct ScopeItemParser;
impl Parse for ScopeItemParser {
    type Result = ScopeItem;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        fn expression_to_scope_item(expr: Expression) -> ScopeItem {
            ScopeItem::Expression(Box::new(expr))
        }
        AssignExprParser
            .or_enum(ScopeItem::AssignExpr)
            .or(ConstExprParser, ScopeItem::ConstExpr)
            .or(LocalExprParser, ScopeItem::LocalExpr)
            .or(ReadExprParser, ScopeItem::ReadExpr)
            .or(UpdateExprParser, ScopeItem::UpdateExpr)
            .or(CountExprParser, ScopeItem::CountExpr)
            .or(ExpressionParser, expression_to_scope_item)
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct LetExprParser;
impl Parse for LetExprParser {
    type Result = LetExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("let")
            .sbws()
            .drop(ScopeItemParser)
            .surrounded_by_rb()
            .map(|scope_item| LetExpr { scope_item })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct IfExprParser;
impl Parse for IfExprParser {
    type Result = IfExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("if").or(str_p("iftc")).sbws()
            .and(ExpressionParser.sbws())
            .and(ExpressionParser.sbws())
            .and(ExpressionParser.optional().sbws())
            .surrounded_by_rb()
            .map(|(((if_or_iftc, predicate), then_branch), else_branch)| {
                let condition_type = match if_or_iftc {
                    Either::Left(_) => ConditionType::If,
                    Either::Right(_) => ConditionType::IfTc
                };

                IfExpr {
                    condition_type,
                    predicate: Box::new(predicate),
                    then_branch: Box::new(then_branch),
                    else_branch: else_branch.map(Box::new),
                }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct LambdaExprParser;
impl Parse for LambdaExprParser {
    type Result = LambdaExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("lambda")
            .sbws()
            .drop(SymbolOrQuotedSymbolParser.sbws().zero_or_more().sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(params, body)| LambdaExpr {
                params,
                body: Box::new(body)
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ApplyExprParser;
impl Parse for ApplyExprParser {
    type Result = ApplyExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("apply")
            .or(str_p("applytc"))
            .or(str_p("applyrec"))
            .sbws()
            .and(ExpressionParser.sbws())
            .and(ExpressionParser.sbws().zero_or_more().sbws())
            .surrounded_by_rb()
            .map(|((apply_type, function_ref), expressions)| {
                let apply_type = match apply_type {
                    Either::Left(aptype) => match aptype {
                        Either::Left(_) => ApplyType::Apply,
                        Either::Right(_) => ApplyType::ApplyTc,
                    },
                    Either::Right(_) => ApplyType::ApplyRec
                };

                ApplyExpr {
                    apply_type,
                    function_ref: Box::new(function_ref),
                    expressions
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct LabelExprParser;
impl Parse for LabelExprParser {
    type Result = LabelExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("label")
            .sbws()
            .drop(SymbolParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                LabelExpr {
                    symbol,
                    expression: Box::new(expression),
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ReturnExprParser;
impl Parse for ReturnExprParser {
    type Result = ReturnExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("return")
            .or(str_p("returntc"))
            .sbws()
            .and(ExpressionParser.sbws().zero_or_more())
            .surrounded_by_rb()
            .map(|(return_type, expression)| {
                let return_type = match return_type {
                    Either::Left(_) => ReturnType::Return,
                    Either::Right(_) => ReturnType::ReturnTc
                };

                ReturnExpr {
                    return_type,
                    expression,
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ListExprParser;
impl Parse for ListExprParser {
    type Result = ListExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("list")
            .sbws()
            .drop(ExpressionParser.sbws().zero_or_more())
            .surrounded_by_rb()
            .map(|expressions| {
                ListExpr {
                    expressions
                }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct HeadExprParser;
impl Parse for HeadExprParser {
    type Result = HeadExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("head")
            .sbws()
            .drop(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|list_ref| {
                HeadExpr {
                    list_ref: Box::new(list_ref),
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct TailExprParser;
impl Parse for TailExprParser {
    type Result = TailExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("tail")
            .sbws()
            .drop(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|list_ref| {
                TailExpr {
                    list_ref: Box::new(list_ref),
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct LengthExprParser;
impl Parse for LengthExprParser {
    type Result = LengthExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("length")
            .sbws()
            .drop(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|list_ref| {
                let list_ref = Box::new(list_ref);
                LengthExpr { list_ref }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct AppendExprParser;
impl Parse for AppendExprParser {
    type Result = AppendExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("append")
            .sbws()
            .drop(ExpressionParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(list_ref1, list_ref2)| {
                let list_ref1 = Box::new(list_ref1);
                let list_ref2 = Box::new(list_ref2);
                AppendExpr { list_ref1, list_ref2 }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct FopenExprParser;
impl Parse for FopenExprParser {
    type Result = FopenExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("fopen")
            .sbws()
            .drop(ExpressionParser.sbws())
            .and(StringParser.sbws())
            .and(StringParser.sbws())
            .surrounded_by_rb()
            .map(|((handle, filename), mode)| {
                let handle = Box::new(handle);
                FopenExpr { handle, filename, mode }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct FcloseExprParser;
impl Parse for FcloseExprParser {
    type Result = FcloseExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("fclose")
            .sbws()
            .drop(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|handle| {
                let handle = Box::new(handle);
                FcloseExpr { handle }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct IoReadExprParser;
impl Parse for IoReadExprParser {
    type Result = IoReadExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("ioread")
            .sbws()
            .drop(ExpressionParser.sbws())
            .and(ExpressionParser.sbws().optional())
            .surrounded_by_rb()
            .map(|(handle, offset)| {
                let handle = Box::new(handle);
                IoReadExpr { handle, offset: offset.map(Box::new) }
            })
            .parse(input)
    }
}

#[derive(Clone, Copy)]
struct IoWriteExprParser;
impl Parse for IoWriteExprParser {
    type Result = IoWriteExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("iowrite")
            .sbws()
            .drop(ExpressionParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(handle, data)| {
                let handle = Box::new(handle);
                let data = Box::new(data);
                IoWriteExpr { handle, data }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct BufExprParser;
impl Parse for BufExprParser {
    type Result = BufExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("buf")
            .sbws()
            .drop(SymbolParser.sbws())
            .and(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|(symbol, expression)| {
                let expression = Box::new(expression);
                BufExpr {
                    symbol,
                    expression
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct IterExprParser;
impl Parse for IterExprParser {
    type Result = IterExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("iterator.loop")
            .sbws()
            .drop(ExpressionParser.sbws())
            .or(str_p("iterator.value").sbws())
            .surrounded_by_rb()
            .map(|loop_or_value| {
                match loop_or_value {
                    Either::Left(x) => { IterExpr::IterLoop(Box::new(x)) },
                    Either::Right(_) => { IterExpr::IterValue }
                }
            })
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct RandExprParser;
impl Parse for RandExprParser {
    type Result = RandExpr;

    fn parse<'b>(&self, input: Input<'b>) -> ParseResult<'b, Self::Result> {
        str_p("rand")
            .sbws()
            .drop(ExpressionParser.sbws())
            .surrounded_by_rb()
            .map(|expression| {
                RandExpr {
                    expression: Box::new(expression),
                }
            })
            .parse(input)
    }
}

fn parse_comments(code: &str) -> (Vec<String>, String) {
    let mut stk = vec![];
    let mut comments = vec![];
    let mut remaining_code = String::new();

    let mut char_iter = code.chars();

    loop {
        let ch = match char_iter.next() {
            Some(ch) => ch,
            None => break
        };

        if ch == '"' {
            if let Some('"') = stk.last() {
                stk.pop();
            } else {
                stk.push('"');
            }
        }
        if stk.is_empty() && ch == ';' {
            let mut comment = String::new();
            while let Some(ch) = char_iter.next() {
                if ch == '\n' { break }
                comment.push(ch);
            }
            comments.push(comment);
        } else {
            remaining_code.push(ch);
        }
    }

    (comments, remaining_code)
}


use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let dir_path = "Gannet-examples/Gannet-examples";
    let parser = SExpressionParser;

//    let program = r#"(let (assign 'ff (lambda 'n 'acc 'f_ '(if (< n '2) '(return acc) '(apply (read 'f_) (- n '1) (* acc n) '(read 'f_))))) (apply (read 'ff) '5 '1 '(read 'ff)))"#;
    //let program = r#"(let '(* '3 '4) '(* '4 '5) '(* '6 '7))"#;
    //dbg!(parser.run(program));
    let mut all_the_code = "".to_string();
    let mut num_success = 0;
    let mut num_failure = 0;

    for entry in fs::read_dir(dir_path)? {
        let entry = entry?;
        let path = entry.path();
        //if !path.ends_with("hello_world.td") { continue }

        if path.is_file() {
            println!("Read File: {}", path.display());
            let code = fs::read_to_string(path)?;
            all_the_code += "\n\n";
            all_the_code += &code;
            println!("=========================================================");
            println!("Input:\n{code:?}");
            let (comments, code) = parse_comments(&code);
            println!("Comments: {comments:?}");
            println!("Remaining: {code:?}");
            match parser.run(&code) {
                Ok(val) => {
                    num_success += 1;
                    println!("Result:\n{val:?}");
                },
                Err(err) => {
                    num_failure += 1;
                    println!("Err:\n{err:?}");
                }
            }
            println!("=========================================================");

        } else if path.is_dir() {
            println!("Directory: {}", path.display());
        }
    }

    println!("Sucess: {num_success}\nFailures: {num_failure}\n");

//     fs::write("all_the_code.td", all_the_code)?;

    Ok(())
}
