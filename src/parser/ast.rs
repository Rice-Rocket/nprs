#[derive(Debug)]
pub enum Statement {
    Assign {
        var: String,
        value: Box<Expr>,
    },
    Pass {
        name: String,
        value: Box<Expr>,
    },
    Edge {
        pass: String,
        dependencies: Vec<String>,
    },
    Display {
        pass: String,
    },
    Error,
}

#[derive(Clone, Debug)]
#[allow(clippy::vec_box)]
pub enum Expr {
    /// An integer.
    Int(i32),
    /// A floating point number.
    Float(f32),
    /// A path to a file.
    Path(String),
    /// A variable access, yielding its value if set and failing if not.
    VarAccess(String),
    Ident(String),
    /// An argument passed in and read from the command line.
    Argument {
        /// The name of the argument as it will be read from the command line.
        name: String,
        /// An optional default value that will be used if no argument is given.
        default: Option<Box<Expr>>,
    },
    /// A tuple struct.
    TupleStruct {
        /// The name of the struct.
        name: String,
        /// The fields of the struct, in order.
        fields: Vec<Box<Expr>>,
    },
    /// A struct or unit struct. Unit structs have no fields.
    Struct {
        /// The name of the struct.
        ///
        /// ```md
        /// **Person** {
        ///     name: "bob", 
        ///     age: 20, 
        ///     ..adult
        /// }
        /// ```
        name: String,
        /// The fields of the struct.
        ///
        /// ```md
        /// Person {
        ///     **name: "bob",**
        ///     **age: 20,**
        ///     ..adult
        /// }
        /// ```
        fields: Vec<Box<Field>>,
        /// The struct updater, if applicable.
        ///
        /// ```md
        /// Person {
        ///     name: "bob",
        ///     age: 20,
        ///     **..adult**
        /// }
        /// ```
        update: Option<String>,
    },
    Error,
}

#[derive(Clone, Debug)]
pub struct Field {
    pub ident: String,
    pub value: Box<Expr>,
}
