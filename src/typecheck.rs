use std::rc::Rc;
use std::collections::HashSet;
use std::fmt;

use ast::{self, Ident, Expr, Literal, ArithBinOp, CmpBinOp, If, Fun, LetFun, LetRec, Apply};
use context::TypeContext;

pub type Result = ::std::result::Result<Type, TypeError>;

#[derive(Debug)]
pub struct TypeError {
    pub message: String,
}

#[derive(PartialEq, Eq, Clone)]
pub enum Type {
    Int,
    Bool,
    Arrow(Rc<Type>, Rc<Type>),
}

use self::Type::*;

impl Type {
    fn maps_to(self, other: Type) -> Type {
        Arrow(Rc::new(self), Rc::new(other))
    }
}

trait IntoType {
    fn as_type(&self) -> Type;
}

impl IntoType for ast::Type {
    fn as_type(&self) -> Type {
        match *self {
            ast::Type::Int => Int,
            ast::Type::Bool => Bool,
            ast::Type::Arrow(ref l, ref r) => Arrow(Rc::new(l.as_type()), Rc::new(r.as_type())),
        }
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Int => f.write_str("int"),
            Bool => f.write_str("bool"),
            Arrow(ref l, ref r) => {
                match **l {
                    Arrow(..) => write!(f, "({:?}) -> {:?}", l, r),
                    _ => write!(f, "{:?} -> {:?}", l, r),
                }
            }
        }
    }
}

pub fn typecheck(expr: &Expr) -> Result {
    let mut ctx = TypeContext::empty();
    expr.check(&mut ctx)
}

macro_rules! bail {
    ($msg:expr) => { bail!($e, $msg,) };

    ($msg:expr, $($farg:expr),*) => {
        return Err(TypeError {
            message: format!($msg $(, $farg)*),
        })
    };
}

fn expect<'c>(expr: &'c Expr, type_: Type, ctx: &mut TypeContext<'c>) -> Result {
    let t = try!(expr.check(ctx));
    if t != type_ {
        bail!("Expected {:?}, got {:?} in {:?}", type_, t, expr);
    }
    Ok(type_)
}

trait Typecheck {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result;
}

impl Typecheck for Expr {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        use ast::Expr::*;
        match *self {
            Var(ref ident) => {
                ctx.lookup(ident)
                   .cloned()
                   .ok_or(TypeError { message: format!("Unbound variable: {}", ident) })
            }
            Literal(ref l) => l.check(ctx),
            ArithBinOp(ref op) => op.check(ctx),
            CmpBinOp(ref op) => op.check(ctx),
            If(ref if_) => if_.check(ctx),
            Fun(ref fun) => fun.check(ctx),
            LetFun(ref let_fun) => let_fun.check(ctx),
            LetRec(ref let_rec) => let_rec.check(ctx),
            Apply(ref apply) => apply.check(ctx),
        }
    }
}

impl Typecheck for Literal {
    fn check<'c>(&'c self, _: &mut TypeContext<'c>) -> Result {
        let t = match *self {
            Literal::Number(_) => Int,
            Literal::Bool(_) => Bool,
        };
        Ok(t)
    }
}

impl Typecheck for ArithBinOp {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        try!(expect(&self.lhs, Int, ctx));
        try!(expect(&self.rhs, Int, ctx));
        Ok(Int)
    }
}

impl Typecheck for CmpBinOp {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        try!(expect(&self.lhs, Int, ctx));
        try!(expect(&self.rhs, Int, ctx));
        Ok(Bool)
    }
}

impl Typecheck for If {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        try!(expect(&self.cond, Bool, ctx));
        let t1 = try!(self.tru.check(ctx));
        let t2 = try!(self.fls.check(ctx));
        if t1 != t2 {
            bail!("Arms of an if have different types: {:?} {:?}", t1, t2);
        }
        Ok(t1)
    }
}

impl Typecheck for Fun {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        let result = fun_type(self);
        try!(ctx.with_bindings(vec![(&self.arg_name, self.arg_type.as_type()),
                                    (&self.fun_name, result.clone())],
                               |ctx| expect(&self.body, self.fun_type.as_type(), ctx)));
        Ok(result)
    }
}

fn fun_type(f: &Fun) -> Type {
    let arg_type = f.arg_type.as_type();
    let ret_type = f.fun_type.as_type();
    arg_type.clone().maps_to(ret_type.clone())
}

impl Typecheck for LetFun {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        let fun_type = try!(self.fun.check(ctx));
        ctx.with_bindings(vec![(&self.fun.fun_name, fun_type)],
                          |ctx| self.body.check(ctx))
    }
}

impl Typecheck for LetRec {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        let bindings = try!(collect_bindings(&self.funs));
        ctx.with_bindings(bindings, |ctx| {
            for fun in &self.funs {
                try!(fun.check(ctx));
            }
            self.body.check(ctx)
        })
    }
}

fn collect_bindings(funs: &[Fun]) -> ::std::result::Result<Vec<(&Ident, Type)>, TypeError> {
    let names = funs.iter().map(|fun| &fun.fun_name).collect::<HashSet<_>>();
    if names.len() != funs.len() {
        return bail!("Duplicate definitions in letrec: {:?}", funs);
    }
    Ok(funs.iter().map(|f| (&f.fun_name, fun_type(f))).collect())
}

impl Typecheck for Apply {
    fn check<'c>(&'c self, ctx: &mut TypeContext<'c>) -> Result {
        match try!(self.fun.check(ctx)) {
            Type::Arrow(arg, ret) => {
                try!(expect(&self.arg, arg.as_ref().clone(), ctx));
                Ok(ret.as_ref().clone())
            }
            _ => return bail!("Not a function {:?}", self.fun),
        }
    }
}

#[cfg(test)]
mod tests {
    use ast::Expr;
    use super::*;
    use super::Type::*;

    fn parse(expr: &str) -> Expr {
        ::syntax::parse(expr).expect(&format!("Failed to parse {}", expr))
    }

    fn assert_valid(expr: &str, type_: Type) {
        let expr = parse(expr);
        match typecheck(&expr) {
            Ok(t) => {
                assert!(t == type_,
                        "Wrong type for {:?}.\nExpected {:?}, got {:?}",
                        expr,
                        type_,
                        t)
            }
            Err(e) => assert!(false, "Failed to typecheck {:?}:\n {:?}", expr, e),
        }
    }

    fn assert_fails(expr: &str) {
        let expr = parse(expr);
        let t = typecheck(&expr);
        assert!(t.is_err(),
                "This expression should not typecheck: {:?}",
                expr);
    }

    #[test]
    fn test_arithmetics() {
        assert_valid("92", Int);
        assert_valid("true", Bool);

        assert_valid("1 + 1", Int);
        assert_fails("1 * true");
    }

    #[test]
    fn test_bools() {
        assert_valid("1 < 1", Bool);
        assert_fails("true == true");
        assert_fails("false > 92");
    }

    #[test]
    fn test_if() {
        assert_valid("if 1 < 2 then 92 else 62", Int);
        assert_valid("if true then false else true", Bool);
        assert_fails("if 1 + (1 == 2) then 92 else 62");
        assert_fails("if 1 then 92 else 62");
        assert_fails("if true then 92 else false");
    }

    #[test]
    fn test_fun() {
        assert_valid("fun id (x: int): int is x", Int.maps_to(Int));
        assert_valid("fun id (x: int): int is id x", Int.maps_to(Int));
        assert_valid("(fun id (x: int): int is x) 92", Int);

        assert_fails("fun id (x: int): int is y");
        assert_fails("(fun id (x: int): int is x) true");
    }

    #[test]
    fn test_let_fun() {
        assert_valid("let fun inc (x: int): int is x + 1 in inc 92", Int);

        assert_fails("let fun inc (x: int): int is x + 1 in inc inc");
    }

    #[test]
    fn test_let_rec() {
        assert_valid("let rec fun a(x: int): int is b (a (b 1))
                      and fun b(x: int): int is (a (b (a 1)))
                      in (a (a (b (b 1))))",
                     Int);

    }
}
