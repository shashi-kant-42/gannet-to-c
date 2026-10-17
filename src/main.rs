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
type VarSymbolParser = SymbolParser::<VarSymbol>;

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
            .and(ServiceExprParser
                .or(SymbolParser(PhantomData)))
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
            .and(SymbolParser(PhantomData))
            .and(ArgExprParser.one_or_more())
            .keep(char_p(')'))
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
            .drop(SymbolParser(PhantomData))
            .keep(char_p(')'))
            .map(|var_symbol| ReadExpr { var_symbol })
            .parse(input)
    }
}


struct GroupExprParser;

impl Parse for GroupExprParser {
    type Result = GroupExpr;

    fn parse<'b>(&self, input: &'b str) -> ParseResult<'b, Self::Result> {
        char_p('(')
            .drop(str_p("group "))
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
            .and(SingleQuoteParser.drop(ServiceExprParser))
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

fn main() {
    dbg!(ServiceExprParser.run("(compress '(compose (img cam1) (img cam3)))"));
    dbg!(ServiceExprParser.run("(encrypt (convert jpeg (img cam1)))"));

        // 1. Space after the inner opening parenthesis
    assert_eq!(
        ServiceExprParser.run("(encrypt (convert 'jpeg (img cam2)))"),
        ServiceExprParser.run("(encrypt ( convert 'jpeg (img cam2)))")
    );

    // 2. Space after the root opening parenthesis
    assert_eq!(
        ServiceExprParser.run("(encrypt (convert jpeg '(img cam2)))"),
        ServiceExprParser.run("( encrypt (convert jpeg '(img cam2)))")
    );

    // 3. Extra spaces around the token 'jpeg'
    assert_eq!(
        ServiceExprParser.run("(encrypt (convert jpeg (img cam2)))"),
        ServiceExprParser.run("(encrypt (convert   jpeg   (img cam2)))")
    );

    // 4. Extra spaces before closing parentheses
    assert_eq!(
        ServiceExprParser.run("(encrypt (convert jpeg (img cam2)))"),
        ServiceExprParser.run("(encrypt (convert jpeg (img cam2) ))")
    );

    // 5. Heavy, scattered whitespace throughout the entire expression
    assert_eq!(
        ServiceExprParser.run("(encrypt (convert jpeg (img cam2)))"),
        ServiceExprParser.run("( encrypt   ( convert   jpeg   ( img   cam2 ) ) )")
    );
}
