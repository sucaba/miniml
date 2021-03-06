use std::fmt;

#[derive(PartialEq, Eq)]
pub enum Type {
    Int,
    Bool,
    Arrow(Box<Type>, Box<Type>),
}

impl Type {
    pub fn arrow(arg: Type, ret: Type) -> Type {
        Type::Arrow(Box::new(arg), Box::new(ret))
    }
}

impl fmt::Debug for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Type::*;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assoc() {
        let foo = Type::arrow(Type::Int, Type::arrow(Type::Bool, Type::Int));
        assert_eq!(format!("{:?}", foo), "int -> bool -> int");

        let foo = Type::arrow(Type::arrow(Type::Int, Type::Bool), Type::Int);
        assert_eq!(format!("{:?}", foo), "(int -> bool) -> int");
    }
}
