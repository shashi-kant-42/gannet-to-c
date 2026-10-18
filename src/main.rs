use std::{fmt::Debug, marker::PhantomData};

use pa_rs::parser::*;

#[derive(Debug, PartialEq, Eq)]
struct OpeningBracket;
#[derive(Debug, PartialEq, Eq)]
struct ClosingBracket;
#[derive(Debug, PartialEq, Eq)]
struct SingleQuote;
#[derive(Debug, PartialEq, Eq)]
struct LiteralSymbol(String);
#[derive(Debug, PartialEq, Eq)]
struct ServiceSymbol(String);

#[derive(Debug, PartialEq, Eq)]
struct ServiceExpr {
    service_symbol: ServiceSymbol,
    arg_expr: Vec<ArgExpr>,
}

#[derive(Debug, PartialEq, Eq)]
enum ArgExpr {
    ServiceExpr(ServiceExpr),
    LiteralSymbol(LiteralSymbol),
    Quoted(Box<ArgExpr>),
}

/*
group-expr::= ’(group ’ “’”? assign-expr+ “’”? arg-expr ’)’
assign-expr::= ’(assign ’ “’” var-symbol arg-expr+ ’)’
read-expr::= ’(read ’ “’” var-symbol ’)’
*/

#[derive(Debug, PartialEq, Eq)]
struct VarSymbol(String);

#[derive(Debug, PartialEq, Eq)]
struct Quoted<T>(T);

#[derive(Debug, PartialEq, Eq)]
enum AssignExpr {
    Expr {
        var_symbol: VarSymbol,
        arg_expr: Vec<ArgExpr>,
    },
    Quoted(Box<AssignExpr>),
}

#[derive(Debug, PartialEq, Eq)]
struct GroupExpr {
    assign_expr: Vec<AssignExpr>,
    arg_expr: ArgExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct ReadExpr {
    var_symbol: VarSymbol,
}

#[derive(Debug, PartialEq, Eq)]
struct IfExpr {
    service_expr: ServiceExpr,
    arg_expr1: ArgExpr,
    arg_expr2: ArgExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct LambdaExpr {
    var_symbols: Vec<VarSymbol>,
    service_expr: ServiceExpr,
}


#[derive(Debug, PartialEq, Eq)]
struct ApplyExpr {
    lambda_expr: LambdaExpr,
    arg_exprs: Vec<ArgExpr>,
}


#[derive(Clone, Copy)]
struct SingleQuoteParser;
impl Parse for SingleQuoteParser {
    type Result = SingleQuote;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('\'')
            .map(|_| SingleQuote)
            .parse(input)
    }
}

#[derive(Debug, PartialEq, Eq)]
struct ListExpr {
    arg_exprs: Vec<ArgExpr>,
}

#[derive(Debug, PartialEq, Eq)]
enum SingleListOp {
    Head,
    Tail,
    Length,
}

#[derive(Debug, PartialEq, Eq)]
struct SingleListExpr {
    single_list_op: SingleListOp,
    list_expr: ListExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct ConsExpr {
    list_expr: ListExpr,
    arg_expr: ArgExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct BufVar(String);

#[derive(Debug, PartialEq, Eq)]
struct BufExpr {
    buf_var: BufVar,
    service_expr: ServiceExpr,
}

#[derive(Debug, PartialEq, Eq)]
struct StreamExpr {
    buf_var: BufVar,
}


#[derive(Debug, PartialEq, Eq)]
struct EosExpr {
    vbuf_var: BufVar,
}

#[derive(Debug, PartialEq, Eq)]
struct PeekExpr {
    buf_var: BufVar,
}

#[derive(Debug, PartialEq, Eq)]
struct GetExpr {
    buf_var: BufVar,
}

trait VarOp {
    fn new(symbol: String) -> Self;
}

impl VarOp for StreamExpr {
    fn new(symbol: String) -> Self {
        StreamExpr { buf_var: BufVar(symbol) }
    }
}

impl VarOp for EosExpr {
    fn new(symbol: String) -> Self {
        EosExpr { vbuf_var: BufVar(symbol) }
    }
}

impl VarOp for PeekExpr {
    fn new(symbol: String) -> Self {
        PeekExpr { buf_var: BufVar(symbol) }
    }
}

impl VarOp for GetExpr {
    fn new(symbol: String) -> Self {
        GetExpr { buf_var: BufVar(symbol) }
    }
}

trait Symbol {
    fn new(symbol: String) -> Self;
}

impl Symbol for LiteralSymbol {
    fn new(symbol: String) -> Self {
        LiteralSymbol(symbol)
    }
}

impl Symbol for ServiceSymbol {
    fn new(symbol: String) -> Self {
        ServiceSymbol(symbol)
    }
}

impl Symbol for VarSymbol {
    fn new(symbol: String) -> Self {
        VarSymbol(symbol)
    }
}

impl Symbol for BufVar {
    fn new(symbol: String) -> Self {
        BufVar(symbol)
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Gannet {
    ServiceExpr(ServiceExpr),
    IfExpr(IfExpr),
    LambdaExpr(LambdaExpr),
    ApplyExpr(ApplyExpr),
    ListExpr(ListExpr),
    SingleListExpr(SingleListExpr),
    ConsExpr(ConsExpr),
    BufExpr(BufExpr),
    StreamExpr(StreamExpr),
    EosExpr(EosExpr),
    PeekExpr(PeekExpr),
    GetExpr(GetExpr),
    AssignExpr(AssignExpr),
    GroupExpr(GroupExpr),
    ReadExpr(ReadExpr),
}

#[derive(Clone, Copy)]
struct SymbolParser<T>(PhantomData<T>);
impl<T: Symbol + Debug + PartialEq> Parse for SymbolParser<T> {
    type Result = T;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        let allowed_chars = ('a'..='z')
            .chain('A'..='Z')
            .chain('0'..='9')
            .chain(['-', '_'].into_iter())
            .map(|ch| char_p(ch));
        one_of_p(allowed_chars)
            .one_or_more()
            .map(|chars| T::new(chars.into_iter().collect()))
            .parse(input)
    }
}


#[derive(Clone, Copy)]
struct ArgExprParser;
impl Parse for ArgExprParser {
    type Result = ArgExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        SingleQuoteParser
            .optional()
            .and(ServiceExprParser.sbws()
                .or(SymbolParser(PhantomData).sbws()))
            .map(|(quote, either)| {
                let mut result = match either {
                    Either::Left(service_expr) => {
                        ArgExpr::ServiceExpr(service_expr)
                    },
                    Either::Right(literal_symbol) => {
                        ArgExpr::LiteralSymbol(literal_symbol)
                    },
                };

                if let Some(_) = quote {
                    result = ArgExpr::Quoted(Box::new(result));
                }
                result
            })
            .parse(input)
    }
}


struct ServiceExprParser;

impl Parse for ServiceExprParser {
    type Result = ServiceExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(').sbws()
            .drop(SymbolParser(PhantomData).sbws())
            .and(ArgExprParser.sbws().one_or_more())
            .keep(char_p(')').sbws())
            .map(|(service_symbol, arg_expr)| {
                ServiceExpr {
                    service_symbol,
                    arg_expr,
                }
            })
            .parse(input)
    }
}

struct AssignExprParser;

impl Parse for AssignExprParser {
    type Result = AssignExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        SingleQuoteParser.optional()
            .keep(char_p('('))
            .keep(str_p("assign"))
            .keep(SingleQuoteParser)
            .and(SymbolParser(PhantomData).sbws())
            .and(ArgExprParser.sbws().one_or_more())
            .keep(char_p(')').sbws())
            .map(|((quote, var_symbol), arg_expr)| {
                let mut result = AssignExpr::Expr { var_symbol, arg_expr };
                if let Some(_) = quote {
                    result = AssignExpr::Quoted(Box::new(result));
                }
                result
            })
            .parse(input)
    }
}


struct ReadExprParser;

impl Parse for ReadExprParser {
    type Result = ReadExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("read"))
            .drop(SingleQuoteParser)
            .drop(SymbolParser(PhantomData).sbws())
            .keep(char_p(')').sbws())
            .map(|var_symbol| ReadExpr { var_symbol })
            .parse(input)
    }
}


struct GroupExprParser;

impl Parse for GroupExprParser {
    type Result = GroupExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("let "))
            .drop(AssignExprParser.sbws().one_or_more())
            .and(ArgExprParser.sbws())
            .keep(char_p(')').sbws())
            .map(|(assign_expr, arg_expr)| {
                GroupExpr {
                    assign_expr,
                    arg_expr,
                }
            })
            .parse(input)
    }
}


struct IfExprParser;

impl Parse for IfExprParser {
    type Result = IfExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("if "))
            .drop(ServiceExprParser.sbws())
            .and(ArgExprParser.sbws())
            .and(ArgExprParser.sbws())
            .keep(char_p(')').sbws())
            .map(|((service_expr, arg_expr1), arg_expr2)| {
                IfExpr {
                    service_expr,
                    arg_expr1,
                    arg_expr2,
                }
            })
            .parse(input)
    }
}

struct LambdaExprParser;

impl Parse for LambdaExprParser {
    type Result = LambdaExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("lambda "))
            .drop(SingleQuoteParser.drop(SymbolParser(PhantomData).sbws()).one_or_more())
            .and(SingleQuoteParser.drop(ServiceExprParser.sbws()))
            .keep(char_p(')').sbws())
            .map(|(var_symbols, service_expr)| {
                LambdaExpr {
                    var_symbols,
                    service_expr,
                }
            })
            .parse(input)
    }
}

struct ApplyExprParser;

impl Parse for ApplyExprParser {
    type Result = ApplyExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("apply "))
            .drop(LambdaExprParser.sbws())
            .and(SingleQuoteParser.drop(ArgExprParser).one_or_more())
            .keep(char_p(')').sbws())
            .map(|(lambda_expr, arg_exprs)| {
                ApplyExpr {
                    lambda_expr,
                    arg_exprs,
                }
            })
            .parse(input)
    }
}


struct ListExprParser;

impl Parse for ListExprParser {
    type Result = ListExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("list "))
            .drop(ArgExprParser.sbws().one_or_more())
            .keep(char_p(')').sbws())
            .map(|arg_exprs| {
                ListExpr {
                    arg_exprs,
                }
            })
            .parse(input)
    }
}

struct SingleListOpParser;

impl Parse for SingleListOpParser {
    type Result = SingleListOp;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        one_of_p([str_p("head"), str_p("tail"), str_p("length")])
            .map(|x| {
                match x.as_str() {
                    "head" => SingleListOp::Head,
                    "tail" => SingleListOp::Tail,
                    "length" => SingleListOp::Length,
                    _ => unreachable!("This code cannot be reached!"),
                }
            })
            .parse(input)
    }
}


struct SingleListExprParser;

impl Parse for SingleListExprParser {
    type Result = SingleListExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(SingleListOpParser.sbws())
            .and(ListExprParser.sbws())
            .map(|(single_list_op, list_expr)| {
                SingleListExpr {
                    single_list_op,
                    list_expr,
                }
            })
            .parse(input)
    }
}

struct ConsExprParser;

impl Parse for ConsExprParser {
    type Result = ConsExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("cons "))
            .drop(ListExprParser.sbws())
            .and(ArgExprParser.sbws())
            .keep(char_p(')').sbws())
            .map(|(list_expr, arg_expr)| {
                ConsExpr {
                    list_expr,
                    arg_expr,
                }
            })
            .parse(input)
    }
}

struct BufExprParser;

impl Parse for BufExprParser {
    type Result = BufExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("buf "))
            .drop(SingleQuoteParser)
            .drop(SymbolParser(PhantomData).sbws())
            .and(SingleQuoteParser.drop(ServiceExprParser.sbws()))
            .keep(char_p(')').sbws())
            .map(|(buf_var, service_expr)| {
                BufExpr {
                    buf_var,
                    service_expr,
                }
            })
            .parse(input)
    }
}


struct VarOpParser<T>(&'static str, PhantomData<T>);
impl<T: VarOp + Debug + PartialEq + Eq> Parse for VarOpParser<T> {
    type Result = T;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p(self.0))
            .drop(SingleQuoteParser)
            .drop(SymbolParser(PhantomData).sbws())
            .keep(char_p(')').sbws())
            .map(|buf_var: BufVar| {
                T::new(buf_var.0)
            })
            .parse(input)
    }
}


struct StreamExprParser;

impl Parse for StreamExprParser {
    type Result = StreamExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        VarOpParser("stream", PhantomData)
            .parse(input)
    }
}

struct EosExprParser;

impl Parse for EosExprParser {
    type Result = EosExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        VarOpParser("eos", PhantomData)
            .parse(input)
    }
}

struct PeekExprParser;

impl Parse for PeekExprParser {
    type Result = PeekExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        VarOpParser("peek", PhantomData)
            .parse(input)
    }
}

struct GetExprParser;

impl Parse for GetExprParser {
    type Result = GetExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        VarOpParser("get", PhantomData)
            .parse(input)
    }
}

struct GannetParser;
impl Parse for GannetParser {
    type Result = Gannet;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        ServiceExprParser
            .or(AssignExprParser)
            .or(IfExprParser)
            .or(LambdaExprParser)
            .or(ApplyExprParser)
            .or(ListExprParser)
            .or(SingleListExprParser)
            .or(ConsExprParser)
            .or(BufExprParser)
            .or(StreamExprParser)
            .or(EosExprParser)
            .or(PeekExprParser)
            .or(GetExprParser)
            .or(ReadExprParser)
            .or(GroupExprParser)
            .map(|either| match either {
                Either::Right(group_expr) => Gannet::GroupExpr(group_expr),
                Either::Left(either) => match either {
                    Either::Right(read_expr) => Gannet::ReadExpr(read_expr),
                    Either::Left(either) => match either {
                        Either::Right(get_expr) => Gannet::GetExpr(get_expr),
                        Either::Left(either) => match either {
                            Either::Right(peek_expr) => Gannet::PeekExpr(peek_expr),
                            Either::Left(either) => match either {
                                Either::Right(eos_expr) => Gannet::EosExpr(eos_expr),
                                Either::Left(either) => match either {
                                    Either::Right(stream_expr) => Gannet::StreamExpr(stream_expr),
                                    Either::Left(either) => match either {
                                        Either::Right(buf_expr) => Gannet::BufExpr(buf_expr),
                                        Either::Left(either) => match either {
                                            Either::Right(cons_expr) => Gannet::ConsExpr(cons_expr),
                                            Either::Left(either) => match either {
                                                Either::Right(single_list_expr) => Gannet::SingleListExpr(single_list_expr),
                                                Either::Left(either) => match either {
                                                    Either::Right(list_expr) => Gannet::ListExpr(list_expr),
                                                    Either::Left(either) => match either {
                                                        Either::Right(apply_expr) => Gannet::ApplyExpr(apply_expr),
                                                        Either::Left(either) => match either {
                                                            Either::Right(lambda_expr) => Gannet::LambdaExpr(lambda_expr),
                                                            Either::Left(either) => match either {
                                                                Either::Right(if_expr) => Gannet::IfExpr(if_expr),
                                                                Either::Left(either) => match either {
                                                                    Either::Right(assign_expr) => Gannet::AssignExpr(assign_expr),
                                                                    Either::Left(service_expr) => Gannet::ServiceExpr(service_expr),
                                                                },
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            })
            .parse(input)
    }
}


use std::fs;
use std::io;

fn main() -> io::Result<()> {
    let dir_path = "Gannet-examples/Gannet-examples";
    let parser = GannetParser;

    let program = r#"(let (assign 'ff (lambda 'n 'acc 'f_ '(if (< n '2) '(return acc) '(apply (read 'f_) (- n '1) (* acc n) '(read 'f_))))) (apply (read 'ff) '5 '1 '(read 'ff)))"#;
    dbg!(parser.run(program));

    // for entry in fs::read_dir(dir_path)? {
    //     let entry = entry?;
    //     let path = entry.path();

    //     if path.is_file() {
    //         println!("Read File: {}", path.display());
    //         let code = fs::read_to_string(path)?;
    //         dbg!(parser.run(&code));

    //     } else if path.is_dir() {
    //         println!("Directory: {}", path.display());
    //     }
    // }

    Ok(())
}
