use proc_macro2::{Group, TokenStream, TokenTree};
use std::fmt::Display;

use crate::var_table::{self, vtable::Value};

pub fn nova_print_value(args: Vec<Value>) {
    for arg in args.iter() {
        match arg {
            Value::Integer(e) => print!("{}", e),
            Value::Float(f) => print!("{:.2}", f),
            Value::Str(s) => print!("{}", s.to_string()),
            Value::Boolean(s) => print!("{}", s),
            _ => (),
        }
    }
}

pub struct NovaModules {
    modules: Vec<(String, fn(Vec<Value>))>,
}

impl NovaModules {
    pub fn new() -> Self {
        let modules = vec![("MOD<stdio>".to_owned(), nova_print_value as fn(Vec<_>))];

        Self { modules }
    }

    pub fn novautil_literal_to_values(el: &TokenTree) -> Result<Value, &'static str> {
        let mut value = Value::Null;
        match el {
            TokenTree::Literal(lit) => {
                if let Ok(e) = lit.to_string().parse::<i64>() {
                    value = Value::Integer(e);
                    return Ok(value);
                }
                if let Ok(e) = lit.to_string().parse::<f64>() {
                    value = Value::Float(e);
                    return Ok(value);
                }
                if let Ok(e) = lit.to_string().parse::<String>() {
                    //parsing single string literal to handle break line
                    let parsed_str =
                        String::from(e.to_owned().replace("\\n", "\n").trim_matches('"'));

                    value = Value::Str(parsed_str.to_owned());
                    return Ok(value);
                }
                Ok(value)
            }
            _ => Err("Error parsing literal"),
        }
    }

    pub fn get_modules(&self) -> &Vec<(String, fn(Vec<Value>))> {
        &self.modules
    }

    // handle module calls check if there is a match between "ident_str" and some Module saved in the vartable
    pub fn handle_module_calls(
        &self,
        ident_str: String,
        vartable: &var_table::vtable::VarTable,
        stream: TokenStream,
    ) {
        for module in self.get_modules() {
            // check if some MOD<> is integrated into the vartable
            let table_option = vartable.get(&module.0);
            if table_option.is_some() {
                let table = table_option.unwrap();
                match table {
                    Value::Module(m) => {
                        for included_function in m.1.iter() {
                            if included_function.eq(&ident_str) {
                                let mut parsed_args: Vec<Value> = Vec::new();
                                for v in stream.clone().into_iter() {
                                    let value = NovaModules::novautil_literal_to_values(&v.clone());
                                    if value.is_ok() {
                                        let mut value = value.unwrap();
                                        let value_copy = value.clone();

                                        // parse variable interpolation
                                        match value_copy {
                                            Value::Str(e) => {
                                                //println!("value : {e}");
                                                if e.contains("var::") {
                                                    let _g = syn::parse_str::<Group>(&format!(
                                                        "({})",
                                                        e
                                                    ))
                                                    .expect("error parsing var");
                                                    let mut resolved_value = vartable
                                                        .parse_string_vars(e.to_owned())
                                                        .expect("error parsing var");

                                                    // removing parentesis after grouping and resolving interpolation
                                                    //resolved_value.remove(resolved_value.len() - 1);
                                                    //resolved_value.remove(0);
                                                    // removing quotes from String
                                                    resolved_value =
                                                        resolved_value.replace("\"", "");

                                                    value = Value::Str(resolved_value);
                                                }
                                            }
                                            _ => (),
                                        }

                                        //println!("New argument of {} with value: {:?} is being parsed", included_function, value);
                                        parsed_args.push(value);
                                    }
                                }
                                module.1(parsed_args);
                            }
                        }
                    }
                    _ => (),
                }
            }
        }
    }
}