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
}

#[derive(Debug)]
#[allow(clippy::vec_box)]
pub enum Expr {
    Int(i32),
    Float(f32),
    Path(String),
    VarAccess(String),
    Ident(String),
    TupleStruct {
        name: String,
        fields: Vec<Box<Expr>>,
    },
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
}

#[derive(Debug)]
pub struct Field {
    pub ident: String,
    pub value: Box<Expr>,
}
