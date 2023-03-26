#![allow(unused)]

#[cfg(test)]
mod tests {
    use anyhow::*;
    use chibiwasm::execution::runtime::Runtime;
    use chibiwasm::execution::value::Value;
    use std::io::{Cursor, Read};
    use std::{fs, path::Path};
    use wabt::{script::*, Features};

    fn into_wasm_value(values: Vec<wabt::script::Value>) -> Vec<Value> {
        values
            .into_iter()
            .map(|a| match a {
                wabt::script::Value::I32(v) => Value::I32(v),
                wabt::script::Value::I64(v) => Value::I64(v),
                wabt::script::Value::F32(v) => Value::F32(v),
                wabt::script::Value::F64(v) => Value::F64(v),
                wabt::script::Value::V128(_) => todo!(),
            })
            .collect()
    }

    fn run_test(spec_file: &str) -> Result<()> {
        let spec = Path::new("./tests/testsuite").join(spec_file);
        let mut file = fs::File::open(spec)?;
        let mut wast = String::new();
        file.read_to_string(&mut wast)?;

        let mut features = Features::new();
        features.enable_all();
        let mut parser = ScriptParser::<f32, f64>::from_source_and_name_with_features(
            wast.as_bytes(),
            spec_file,
            features,
        )?;

        let mut runtime = {
            if let Some(command) = parser.next()? {
                match command.kind {
                    CommandKind::Module { module, .. } => {
                        let mut reader = Cursor::new(module.into_vec());
                        Runtime::from_reader(&mut reader)?
                    }
                    _ => panic!("not found module"),
                }
            } else {
                panic!("not found any command");
            }
        };

        while let Some(command) = parser.next()? {
            match command.kind {
                CommandKind::AssertReturn { action, expected } => match action {
                    Action::Invoke { field, args, .. } => {
                        let args = into_wasm_value(args);
                        let result = runtime.call(field.clone(), args.clone())?;
                        if result.is_none() {
                            continue;
                        }
                        let got = match result.unwrap() {
                            Value::I32(v) => {
                                vec![wabt::script::Value::I32(v)]
                            }
                            Value::I64(v) => {
                                vec![wabt::script::Value::I64(v)]
                            }
                            Value::F32(v) => {
                                if v.is_nan() {
                                    vec![wabt::script::Value::F32(0_f32)]
                                } else {
                                    vec![wabt::script::Value::F32(v)]
                                }
                            }
                            Value::F64(v) => {
                                if v.is_nan() {
                                    vec![wabt::script::Value::F64(0_f64)]
                                } else {
                                    vec![wabt::script::Value::F64(v)]
                                }
                            }
                        };

                        let want: Vec<_> = expected
                            .into_iter()
                            .map(|e| match e {
                                wabt::script::Value::F32(v) => {
                                    if v.is_nan() {
                                        return wabt::script::Value::F32(0_f32);
                                    }
                                    e
                                }
                                wabt::script::Value::F64(v) => {
                                    if v.is_nan() {
                                        return wabt::script::Value::F64(0_f64);
                                    }
                                    e
                                }
                                _ => e,
                            })
                            .collect();

                        assert_eq!(
                                    want,
                                    got,
                                    "unexpect result, want={want:?}, got={got:?}, test: {field}, args: {args:?}",
                                );
                        //assert_eq!(expected, result, "args: {:?}", args);
                    }
                    Action::Get { .. } => todo!(),
                },
                CommandKind::AssertReturnCanonicalNan { .. } => {
                    // TODO
                }
                CommandKind::AssertReturnArithmeticNan { .. } => {
                    // TODO
                }
                CommandKind::AssertTrap { action, message } => match action {
                    Action::Invoke { field, args, .. } => {
                        let args = into_wasm_value(args);
                        let result = runtime.call(field.clone(), args.clone());

                        match result {
                            Err(err) => {
                                let want = message;
                                let got = err.to_string();
                                assert_eq!(
                                    want,
                                    got,
                                    "unexpect result, want={want}, got={got}, test: {field}, args: {args:?}",
                                );
                            }
                            _ => {
                                panic!("test must be fail: {}", field);
                            }
                        }
                    }
                    Action::Get { .. } => todo!(),
                },
                CommandKind::AssertInvalid { .. } => {
                    // TODO
                }
                CommandKind::AssertMalformed { .. } => {
                    // TODO
                }
                CommandKind::AssertUninstantiable { .. } => {
                    // TODO
                }
                CommandKind::AssertExhaustion { .. } => {
                    // TODO
                }
                CommandKind::AssertUnlinkable { .. } => {
                    // TODO
                }
                CommandKind::Register { .. } => {
                    // TODO
                }
                CommandKind::PerformAction(_) => {
                    // TODO
                }
                _ => {
                    panic!("unexpect command kind: {:?}", command.kind);
                }
            }
        }
        Ok(())
    }

    macro_rules! test {
        ($ty: ident) => {
            #[test]
            fn $ty() -> Result<()> {
                let file = format!("{}.wast", stringify!($ty));
                run_test(&file)?;
                Ok(())
            }
        };
    }

    test!(i32);
    test!(i64);
    test!(f32);
    test!(f32_cmp);
    test!(f32_bitwise);
    test!(f64);
    test!(f64_cmp);
    test!(f64_bitwise);
}
