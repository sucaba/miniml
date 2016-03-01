use super::Type;
use std::fmt::{self, Write};

pub type Ident = String;

pub enum Expr {
    Var(Ident),
    Literal(Literal),
    ArithBinOp(ArithBinOp),
    CmpBinOp(CmpBinOp),
    If(If),
    Fun(Fun),
    Apply(Apply),
}

impl fmt::Debug for Expr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Expr::*;
        match *self {
            Var(ref s) => f.write_str(s),
            Literal(ref l) => l.fmt(f),
            ArithBinOp(ref op) => op.fmt(f),
            CmpBinOp(ref op) => op.fmt(f),
            If(ref if_) => if_.fmt(f),
            Apply(ref apply) => apply.fmt(f),
            Fun(ref fun) => fun.fmt(f),
        }
    }
}

pub struct BinOp<T> {
    pub kind: T,
    pub lhs: Box<Expr>,
    pub rhs: Box<Expr>,
}

impl<T: fmt::Debug> fmt::Debug for BinOp<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
       write!(f, "({:?} {:?} {:?})", self.kind, self.lhs, self.rhs)
    }
}

pub enum ArithOp { Mul, Div, Add, Sub }

impl fmt::Debug for ArithOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ArithOp::*;
        f.write_char(match *self {
            Mul => '*', Div => '\\',
            Add => '+', Sub => '-',
         })
    }
}

pub type ArithBinOp = BinOp<ArithOp>;

pub enum CmpOp { Eq, Lt, Gt }

impl fmt::Debug for CmpOp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::CmpOp::*;
        f.write_str(match *self {
            Eq => "==",
            Lt => "<",
            Gt => ">",
         })
    }
}

pub type CmpBinOp = BinOp<CmpOp>;

pub struct If {
    pub cond: Box<Expr>,
    pub tru: Box<Expr>,
    pub fls: Box<Expr>,
}

impl fmt::Debug for If {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(if {:?} {:?} {:?})", self.cond, self.tru, self.fls)
    }
}

pub struct Fun {
    pub name: Ident,
    pub arg_name: Ident,
    pub arg_type: Box<Type>,
    pub fun_type: Box<Type>,
    pub body: Box<Expr>,
}

impl fmt::Debug for Fun {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "(λ {} ({}: {:?}): {:?} {:?})",
               self.name, self.arg_name, self.arg_type, self.fun_type, self.body)
    }
}

pub struct Apply {
    pub fun: Box<Expr>,
    pub arg: Box<Expr>,
}

impl fmt::Debug for Apply {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({:?} {:?})", self.fun, self.arg)
    }
}

pub enum Literal {
    Number(i64),
    Bool(bool),
}

impl fmt::Debug for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Literal::Number(x) => x.fmt(f),
            Literal::Bool(b) => b.fmt(f),
        }
    }
}
